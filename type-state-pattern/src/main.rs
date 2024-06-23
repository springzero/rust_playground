use std::marker::PhantomData;

struct Player<S>
where
    S: PersonStatus,
{
    name: String,
    health: u8,
    _marker: PhantomData<S>,
}

impl Player<Alive> {
    pub fn new(name: String) -> Self {
        Player {
            name: name,
            health: 100,
            _marker: PhantomData,
        }
    }
}

struct Alive {}
struct Dead {}
impl PersonStatus for Alive {}
impl PersonStatus for Dead {}

trait PersonStatus {}

impl Player<Alive> {
    pub fn show(&self) {
        println!("I am alive, {:?}, health: {:?}", self.name, self.health);
    }

    pub fn die(self) -> Player<Dead> {
        Player {
            name: self.name,
            health: 0,
            _marker: PhantomData,
        }
    }
}

impl Player<Dead> {
    pub fn show(&self) {
        println!("I am Dead, {:?}, health: {:?}", self.name, self.health);
    }
}

fn main() {
    println!("Hello, world!");
    let player = Player::new("one".to_string());
    player.show();
    let player = player.die();
    player.show();
}
