use rand::prelude::*;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

const N_PHILOSOPHERS: usize = 5;

fn main() {
    let (tx, rx) = mpsc::channel();

    let forks: Vec<Arc<Mutex<Fork>>> = (0..N_PHILOSOPHERS)
        .map(|_| Arc::new(Mutex::new(Fork)))
        .collect();

    let mut philosophers = vec![];
    for i in 1..N_PHILOSOPHERS + 1 {
        let left_fork = forks[(i - 1) % N_PHILOSOPHERS].clone();
        let right_fork = forks[i % N_PHILOSOPHERS].clone();
        let philosopher = Philosopher::new(
            i,
            left_fork.clone(),
            right_fork.clone(),
            tx.clone(),
        );
        philosophers.push(philosopher);
    }

    for mut philosopher in philosophers {
        let _ = std::thread::spawn(move || {
            philosopher.run();
        });
    }

    for msg in rx.iter() {
        println!("PHILOSOPHER {} HAS STARVED TO DEATH!", msg);
        break;
    }
}

struct Fork;

struct Philosopher {
    id: usize,
    state: PhilosopherState,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    death_signal: Sender<usize>,
}

#[derive(Eq, PartialEq)]
enum PhilosopherState {
    Eating,
    Hungry(Instant),
    Thinking,
    Dead,
}

impl Philosopher {
    // Maximum number of milliseconds a philosopher can think or eat for
    const MAX_DURATION_MILLIS: u64 = 1000;
    // Minimum number of milliseconds a philosopher can think or eat for
    const MIN_DURATION_MILLIS: u64 = Self::MAX_DURATION_MILLIS / 10;
    // Philosopher will die if they're hungry for longer than this time (milliseconds)
    const HUNGER_THRESHOLD_MILLIS: u128 = Self::MAX_DURATION_MILLIS as u128 * 5;

    pub fn new(
        id: usize,
        left_fork: Arc<Mutex<Fork>>,
        right_fork: Arc<Mutex<Fork>>,
        death_signal: Sender<usize>,
    ) -> Self {
        Self {
            id,
            state: PhilosopherState::Thinking,
            left_fork,
            right_fork,
            death_signal,
        }
    }

    /// Begin the cycle of thinking and eating. Only ends if the philosopher
    /// starves to death.
    pub fn run(&mut self) {
        while self.state != PhilosopherState::Dead {
            self.think();
            self.eat();
        }
    }

    fn think(&mut self) {
        println!("Philosopher {} is thinking", self.id);
        self.state = PhilosopherState::Thinking;
        sleep(Self::generate_duration());

        println!("Philosopher {} is hungry", self.id);
        self.state = PhilosopherState::Hungry(Instant::now());
    }

    fn eat(&mut self) {
        while let PhilosopherState::Hungry(_) = self.state {
            // Attempt to pick up both forks at the same time
            let pickup_forks =
                (self.left_fork.try_lock(), self.right_fork.try_lock());
            if let (Ok(_), Ok(_)) = pickup_forks {
                // Philosopher has successfully picked up both forks and will
                // start to eat.
                println!("Philosopher {} is eating", self.id);
                self.state = PhilosopherState::Eating;
                sleep(Self::generate_duration());
                println!("Philosopher {} is full", self.id);
            } else if self.is_dead() {
                // Philosopher is hungry but could not pick up both forks, so
                // we check if philosopher has starved to death
                self.state = PhilosopherState::Dead;
                self.death_signal
                    .send(self.id)
                    .expect("Failed to send death signal");
            }
        }
    }

    // Determine whether or not a philosopher has starved to death by checking
    // how long they've been hungry for.
    fn is_dead(&self) -> bool {
        if let PhilosopherState::Hungry(hungry_since) = self.state {
            if hungry_since.elapsed().as_millis()
                > Self::HUNGER_THRESHOLD_MILLIS
            {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn generate_duration() -> Duration {
        let millis: u64 = thread_rng()
            .gen_range(Self::MIN_DURATION_MILLIS..Self::MAX_DURATION_MILLIS);
        Duration::from_millis(millis)
    }
}
