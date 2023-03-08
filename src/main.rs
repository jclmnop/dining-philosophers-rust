mod two_forks;
mod sequential;
mod resource_hierarchy;
mod semaphores;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, TryRecvError};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

const N_PHILOSOPHERS: usize = 5;

// Maximum number of milliseconds a philosopher can think or eat for
const MAX_DURATION_MILLIS: u64 = 10;
// Minimum number of milliseconds a philosopher can think or eat for
#[allow(dead_code)]
const MIN_DURATION_MILLIS: u64 = MAX_DURATION_MILLIS / 10;
// Philosopher will die if they're hungry for longer than this time (milliseconds)
const HUNGER_THRESHOLD_MILLIS: u128 = MAX_DURATION_MILLIS as u128 * 10;

const RUN_TIME_SECONDS: u64 = 10;

fn main() {
    println!("~~SEQUENTIAL (CONTROL)~~");
    run(sequential::main);

    println!("\n~~AKIMBO FORKS~~");
    run(two_forks::main);

    println!("\n~~DIJKSTRA'S SEMAPHORES~~");
    run(semaphores::main);

    println!("\n~~RESOURCE HIERARCHY~~");
    run(resource_hierarchy::main);
}

fn run<F>(main_f: F)
where
    F: Send + Fn(Sender<StateMsg>, Arc<AtomicBool>) -> () + 'static,
{
    let (tx, rx) = mpsc::channel::<StateMsg>();
    let killswitch = Arc::new(AtomicBool::new(false));
    let cloned_killswitch = killswitch.clone();
    let main_handle = thread::spawn(move || {
        main_f(tx, cloned_killswitch);
    });
    let start_time = Instant::now();
    let mut meals_eaten = [0; N_PHILOSOPHERS];

    'main_loop: while start_time.elapsed().as_secs() < RUN_TIME_SECONDS {
        match rx.try_recv() {
            Ok(msg) => match msg {
                StateMsg {
                    id,
                    state: PhilosopherState::Eating,
                } => {
                    meals_eaten[id - 1] += 1;
                }
                StateMsg {
                    id,
                    state: PhilosopherState::Dead,
                } => {
                    println!("Philosopher {id} has died from starvation!");
                    return
                }
                _ => {
                    println!("POOP");
                }
            },
            Err(TryRecvError::Disconnected) => {
                println!("Oh no!");
                return
            },
            Err(TryRecvError::Empty) => {}
        }
    }

    killswitch.store(true, Ordering::Relaxed);
    main_handle.join().unwrap();
    println!("\tTotal meals eaten: {}", meals_eaten.iter().sum::<i32>());
    for (i, n) in meals_eaten.iter().enumerate() {
        println!("\tPhilosopher {}: {n} meals", i + 1);
    }
}

pub struct StateMsg {
    pub id: usize,
    pub state: PhilosopherState,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PhilosopherState {
    Eating,
    Hungry(Instant),
    Thinking,
    Dead,
}

pub trait Diner {
    /// Begin the cycle of thinking and eating. Only ends if the philosopher
    /// starves to death or if the killswitch is activated.
    fn run(&mut self) {
        while self.current_state() != PhilosopherState::Dead
            && !self.is_killswitch_active()
        {
            self.think();
            self.eat();
        }
    }

    fn send_state(&self);

    fn think(&mut self);

    fn eat(&mut self);

    fn current_state(&self) -> PhilosopherState;

    fn is_killswitch_active(&self) -> bool;

    /// Has the philosopher been hungry for longer than the maximum time?
    fn has_starved_to_death(&self) -> bool {
        if let PhilosopherState::Hungry(hungry_since) = self.current_state() {
            if hungry_since.elapsed().as_millis() > HUNGER_THRESHOLD_MILLIS {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Eat or think for a random amount of time.
    fn sleep(&self) {
        thread::sleep(Self::generate_duration());
    }

    fn generate_duration() -> Duration {
        // Currently disabled to reduce randomness for benchmarks
        // let millis: u64 = thread_rng()
        //     .gen_range(MIN_DURATION_MILLIS..MAX_DURATION_MILLIS);
        let millis = MAX_DURATION_MILLIS;
        Duration::from_millis(millis)
    }
}
