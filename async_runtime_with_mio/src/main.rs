mod executor {
    use std::{pin::Pin, sync::{mpsc, Arc, Mutex}, task::{Context, Poll, RawWaker, RawWakerVTable}};

    pub trait Future {
        type Output;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
    }

    pub(crate) struct Task {
        future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
        spawner: Spawner,
    }

    impl Task {
        const WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    }

    // fn clone(ptr: *const ()) -> RawWaker {
    //     let ori: Arc<Task> = unsafe { Arc::from_raw(ptr as _)};

    //     // Increment the inner counter of the arc.
    //     let cloned = ori.clone();

    //     std::mem::forget(ori);
    //     std::mem::forget(cloned);

    //     RawWaker::new(ptr, &Task::WAKER_VTABLE)
    // }

    fn drop(ptr: *const ()) {
        let _: Arc<Task> = unsafe { Arc::from_raw(ptr as _)};
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

    pub struct Executor {
        ready_queue: std::sync::mpsc::Receiver<Arc<Task>>,
    }

    impl Executor {
        pub fn run(&self) {
            while let Ok(task) = self.ready_queue.recv() {
                let mut future = task.future.lock().unwrap();

                // make a context (explained later)
                // let waker = Arc::clone(&task).waker();
                // let mut context = Context::from_waker(&waker);

                // // allow the future some CPU time to make progress
                // let _ = future.as_mut().poll(&mut context);
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

        (Executor{ ready_queue }, Spawner { task_sender })
    }

}

fn main() {
    println!("Hello, world!");
}
