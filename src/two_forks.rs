#![allow(unused_imports)]
use crate::{
    Diner, PhilosopherState, StateMsg, HUNGER_THRESHOLD_MILLIS,
    MAX_DURATION_MILLIS, MIN_DURATION_MILLIS, N_PHILOSOPHERS,
};
use rand::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::time::Instant;

/// This one is my solution. The philosophers attempt to pick up both forks,
/// and if they're unable to pick up both they drop any fork they did manage
/// to pick up.
pub fn main(tx: Sender<StateMsg>, kill_switch: Arc<AtomicBool>, random: bool) {
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
            kill_switch.clone(),
            random,
        );
        philosophers.push(philosopher);
    }

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for mut philosopher in philosophers {
        let handle = std::thread::spawn(move || {
            philosopher.run();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

struct Fork;

struct Philosopher {
    id: usize,
    state: PhilosopherState,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    tx: Sender<StateMsg>,
    kill_switch: Arc<AtomicBool>,
    random: bool,
}

impl Philosopher {
    pub fn new(
        id: usize,
        left_fork: Arc<Mutex<Fork>>,
        right_fork: Arc<Mutex<Fork>>,
        tx: Sender<StateMsg>,
        kill_switch: Arc<AtomicBool>,
        random: bool,
    ) -> Self {
        Self {
            id,
            state: PhilosopherState::Thinking,
            left_fork,
            right_fork,
            tx,
            kill_switch,
            random,
        }
    }
}

impl Diner for Philosopher {
    fn send_state(&self) {
        self.tx
            .send(StateMsg {
                id: self.id,
                state: self.current_state(),
            })
            .expect("Error when sending state.");
    }

    fn think(&mut self) {
        log::debug!("Philosopher {} is thinking", self.id);
        self.state = PhilosopherState::Thinking;
        self.sleep(self.random);

        log::debug!("Philosopher {} is hungry", self.id);
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
                log::debug!("Philosopher {} is eating", self.id);
                self.state = PhilosopherState::Eating;
                self.sleep(self.random);
                self.send_state();
                log::debug!("Philosopher {} is full", self.id);
            } else if self.has_starved_to_death() {
                // Philosopher is hungry but could not pick up both forks, so
                // we check if philosopher has starved to death
                self.state = PhilosopherState::Dead;
                self.send_state();
            }
        }
    }

    fn current_state(&self) -> PhilosopherState {
        self.state
    }

    fn is_kill_switch_active(&self) -> bool {
        self.kill_switch.load(Ordering::Relaxed)
    }
}
