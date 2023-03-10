use crate::{Diner, PhilosopherState, StateMsg, N_PHILOSOPHERS};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

const PHILOSOPHER_ORDER: [usize; N_PHILOSOPHERS] = [1, 3, 5, 2, 4];

/// Essentially a 'control' to compare other solutions against.
/// Sequential implementation that just goes around the table in a for loop
/// telling the philosophers to eat if they're hungry and can pick up both forks.
///
/// To be honest, I found this more difficult than doing it in a "normal" way.
pub fn main(tx: Sender<StateMsg>, kill_switch: Arc<AtomicBool>, random: bool) {
    let forks: Vec<Arc<Mutex<Fork>>> = (0..N_PHILOSOPHERS)
        .map(|_| Arc::new(Mutex::new(Fork)))
        .collect();

    let mut philosophers = vec![];
    let mut philosopher_cmd_txs: Vec<SyncSender<PhilosopherCommand>> = vec![];

    for i in 1..N_PHILOSOPHERS + 1 {
        let left_fork = forks[(i - 1) % N_PHILOSOPHERS].clone();
        let right_fork = forks[i % N_PHILOSOPHERS].clone();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::sync_channel(0);
        let philosopher = Philosopher::new(
            i,
            left_fork.clone(),
            right_fork.clone(),
            tx.clone(),
            kill_switch.clone(),
            cmd_rx,
            random,
        );
        philosophers.push(philosopher);
        philosopher_cmd_txs.push(cmd_tx);
    }

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for mut philosopher in philosophers {
        let handle = std::thread::spawn(move || {
            philosopher.run();
        });
        handles.push(handle);
    }

    // Run the sequential loop until kill_switch is active
    while !kill_switch.load(Ordering::Relaxed) {
        for i in &PHILOSOPHER_ORDER {
            let cmd_tx = &philosopher_cmd_txs[i - 1];
            if kill_switch.load(Ordering::Relaxed) {
                cmd_tx.send(PhilosopherCommand::Stop).unwrap();
            } else {
                // Sync channel has buffer size of 0 so sending blocks while
                // philosopher is thinking or waiting to eat.
                match cmd_tx.send(PhilosopherCommand::Eat) {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

struct Fork;

enum PhilosopherCommand {
    Eat,
    Stop,
}

struct Philosopher {
    id: usize,
    state: PhilosopherState,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    tx: Sender<StateMsg>,
    kill_switch: Arc<AtomicBool>,
    cmd_rx: Receiver<PhilosopherCommand>,
    random: bool,
}

impl Philosopher {
    pub fn new(
        id: usize,
        left_fork: Arc<Mutex<Fork>>,
        right_fork: Arc<Mutex<Fork>>,
        tx: Sender<StateMsg>,
        kill_switch: Arc<AtomicBool>,
        cmd_rx: Receiver<PhilosopherCommand>,
        random: bool,
    ) -> Self {
        Self {
            id,
            state: PhilosopherState::Thinking,
            left_fork,
            right_fork,
            tx,
            kill_switch,
            cmd_rx,
            random,
        }
    }
}

impl Diner for Philosopher {
    fn run(&mut self) {
        self.think();
        while !self.is_kill_switch_active() && !self.has_starved_to_death() {
            let command = self.cmd_rx.recv().unwrap();
            match command {
                PhilosopherCommand::Eat => {
                    self.eat();
                }
                PhilosopherCommand::Stop => break,
            }
        }

        if self.has_starved_to_death() {
            self.state = PhilosopherState::Dead;
            self.send_state();
        }
    }

    fn send_state(&self) {
        self.tx
            .send(StateMsg {
                id: self.id,
                state: self.current_state(),
            })
            .expect("Error when sending state.");
    }

    fn think(&mut self) {
        match self.state {
            PhilosopherState::Hungry(_) => {}
            _ => {
                log::debug!("Philosopher {} is thinking", self.id);
                self.state = PhilosopherState::Thinking;
                self.sleep(self.random);

                log::debug!("Philosopher {} is hungry", self.id);
                self.state = PhilosopherState::Hungry(Instant::now());
            }
        }
    }

    fn eat(&mut self) {
        let mut eaten = false;
        match self.state {
            PhilosopherState::Hungry(_) if !self.has_starved_to_death() => {
                while !eaten {
                    let pickup_forks =
                        (self.left_fork.try_lock(), self.right_fork.try_lock());
                    if let (Ok(_), Ok(_)) = pickup_forks {
                        // Philosopher has successfully picked up both forks and will
                        // start to eat.
                        log::debug!("Philosopher {} is eating", self.id);
                        self.state = PhilosopherState::Eating;
                        self.sleep(self.random);
                        self.send_state();
                        log::debug!("Philosopher {} is full", self.id);
                        eaten = true;
                    }
                }
            }
            _ => {}
        }

        if eaten {
            self.think();
        }
    }

    fn current_state(&self) -> PhilosopherState {
        self.state
    }

    fn is_kill_switch_active(&self) -> bool {
        self.kill_switch.load(Ordering::Relaxed)
    }
}
