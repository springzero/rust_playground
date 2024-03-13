mod executor {
    use std::{
        collections::{hash_map::Entry, HashMap}, io::{self, ErrorKind}, net::{SocketAddr, ToSocketAddrs}, pin::Pin, sync::{mpsc, Arc, Mutex, OnceLock}, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
    };

    use mio::{Interest, Registry, Token};

    pub trait Future {
        type Output;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
    }

    pub(crate) struct Task {
        future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
        spawner: Spawner,
    }

    fn clone(ptr: *const ()) -> RawWaker {
        let ori: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };

        // Increment the inner counter of the arc.
        let cloned = ori.clone();

        std::mem::forget(ori);
        std::mem::forget(cloned);

        RawWaker::new(ptr, &Task::WAKER_VTABLE)
    }

    fn drop(ptr: *const ()) {
        let _: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };
    }

    fn wake(ptr: *const ()) {
        let arc: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };
        let spawner = arc.spawner.clone();

        spawner.spawn_task(arc);
    }

    fn wake_by_ref(ptr: *const ()) {
        let arc: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };

        arc.spawner.spawn_task(arc.clone());

        // we don't actually have ownership of this arc value
        // therefore we must not drop `arc`
        std::mem::forget(arc)
    }

    impl Task {
        const WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

        /*
        Why is all this unsafe pointer business required here? It looks like the code could use a Wake trait instead
        Because how the Wake trait cannot be turned into an object, due to the fact that .clone() returns Self. It also gives the following hint:
        note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically
        And that concludes the reason why wakers require a manual vtable. The requirement of erased types combined with a Clone bound make it impossible to use a more standard trait-based approach
        */
        pub fn waker(self: Arc<Self>) -> Waker {
            let opaque_ptr = Arc::into_raw(self) as *const ();
            let vtable = &Self::WAKER_VTABLE;

            unsafe { Waker::from_raw(RawWaker::new(opaque_ptr, vtable)) }
        }
    }

    pub enum Status {
        Awaited(Waker),
        Happened,
    }

    pub struct Reactor {
        registry: Registry,
        statuses: Mutex<HashMap<Token, Status>>,
    }

    impl Reactor {
        pub fn get() -> &'static Self {
            static REACTOR: OnceLock<Reactor> = OnceLock::new();

            REACTOR.get_or_init(|| {
                let poll = mio::Poll::new().unwrap();
                let reactor = Reactor {
                    registry: poll.registry().try_clone().unwrap(),
                    statuses: Mutex::new(HashMap::new()),
                };

                std::thread::Builder::new()
                    .name("reactor".to_owned())
                    .spawn(|| run(poll))
                    .unwrap();

                reactor
            })
        }
    }

    fn run(mut poll: mio::Poll) -> ! {
        let reactor = Reactor::get();
        let mut events = mio::Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in &events {
                let mut guard = reactor.statuses.lock().unwrap();

                let previous = guard.insert(event.token(), Status::Happened);

                if let Some(Status::Awaited(waker)) = previous {
                    waker.wake();
                }
            }
        }
    }

    pub struct Executor {
        ready_queue: std::sync::mpsc::Receiver<Arc<Task>>,
    }

    impl Executor {
        pub fn run(&self) {
            while let Ok(task) = self.ready_queue.recv() {
                let mut future = task.future.lock().unwrap();

                // make a context (explained later)
                let waker = Arc::clone(&task).waker();
                let mut context = Context::from_waker(&waker);

                // allow the future some CPU time to make progress
                let _ = future.as_mut().poll(&mut context);
            }
        }
    }

    #[derive(Clone)]
    pub struct Spawner {
        task_sender: std::sync::mpsc::SyncSender<Arc<Task>>,
    }

    impl Spawner {
        pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
            let task = Arc::new(Task {
                future: Mutex::new(Box::pin(future)),
                spawner: self.clone(),
            });
            self.spawn_task(task);
        }

        pub(crate) fn spawn_task(&self, task: Arc<Task>) {
            self.task_sender.send(task).unwrap();
        }
    }

    pub fn new_executor_spawner() -> (Executor, Spawner) {
        const MAX_QUEUED_TASKS: usize = 10_000;

        let (task_sender, ready_queue) = mpsc::sync_channel(MAX_QUEUED_TASKS);

        (Executor { ready_queue }, Spawner { task_sender })
    }

    // async udpsocket
    pub struct UpdSocket {
        socket: mio::net::UdpSocket,
        token: Token,
    }

    impl Reactor {
        fn unique_token(&self) -> Token {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static CURRENT_TOKEN: AtomicUsize = AtomicUsize::new(0);
            Token(CURRENT_TOKEN.fetch_add(1, Ordering::Relaxed))
        }
    }

    impl UpdSocket {
        pub fn bind(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
            let std_socket = std::net::UdpSocket::bind(addr)?;
            std_socket.set_nonblocking(true)?;

            let mut socket = mio::net::UdpSocket::from_std(std_socket);

            let reactor = Reactor::get();
            let token = reactor.unique_token();

            Reactor::get().registry.register(
                &mut socket,
                token,
                Interest::READABLE | Interest::WRITABLE,
            )?;

            Ok(self::UpdSocket { socket, token })
        }

        pub async fn send_to(&self, buf: &[u8], dest: SocketAddr) -> std::io::Result<usize> {
            loop {
                match self.socket.send_to(buf, dest) {
                    Ok(value) => return Ok(value),
                    Err(error) => {
                        if error.kind() == ErrorKind::WouldBlock {
                            std::future::poll_fn(|cx| {
                                Reactor::get().poll(self.token, cx)
                            }).await?
                        } else {
                            return Err(error),
                        }
                    }
                }
            }
        }
    }

    impl Reactor {
        pub fn poll(&self, token: Token, cx: &mut Context) -> Poll<io::Result<()>> {
            let mut guard = self.statuses.lock().unwrap();
            match guard.entry(token) {
                // If there was no status inserted previously, we simply store the waker, so that the run function will respawn the future when the event happens.
                Entry::Vacant(vacant) => {
                    vacant.insert(Status::Awaited(cx.waker().clone()));
                    Poll::Pending
                }
                // 
                Entry::Occupied(mut occupied) => {
                    match occupied.get() {
                        Status::Awaited(waker) => {
                            // skip clone is wakers are the same
                            if !waker.will_wake(cx.waker()) {
                                occupied.insert(Status::Awaited(cx.waker().clone()));
                            }
                            Poll::Pending
                        }
                        Status::Happened => {
                            occupied.remove();
                            Poll::Ready(Ok(()))
                        }
                    }
                }
            }
        }
    }

}

fn main() {
    println!("Hello, world!");
}
