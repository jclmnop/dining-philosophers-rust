#![allow(unused_imports)]
use crate::{
    Diner, PhilosopherState, StateMsg, HUNGER_THRESHOLD_MILLIS,
    MAX_DURATION_MILLIS, MIN_DURATION_MILLIS, N_PHILOSOPHERS,
};
use rand::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::time::Instant;

/// Based on some other guy's solution, works by assigning a strict ordering
/// hierarchy to the forks. Philosophers will pick up the lowest fork first.
/// Fork order is based on address in memory.
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
            // Pick up left fork first if its address in memory is lower than
            // the right fork.
            let (_left, _right): (MutexGuard<Fork>, MutexGuard<Fork>) =
                if &self.left_fork as *const _ < &self.right_fork as *const _ {
                    let left = self.left_fork.lock();
                    let right = self.right_fork.lock();
                    (left.unwrap(), right.unwrap())
                } else {
                    let right = self.right_fork.lock();
                    let left = self.left_fork.lock();
                    (left.unwrap(), right.unwrap())
                };
            if !self.has_starved_to_death() {
                // Philosopher has successfully picked up both forks and will
                // start to eat, as long as they're not dead.
                log::debug!("Philosopher {} is eating", self.id);
                self.state = PhilosopherState::Eating;
                self.sleep(self.random);
                self.send_state();
                log::debug!("Philosopher {} is full", self.id);
            } else {
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

