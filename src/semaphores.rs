use crate::StateMsg;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;

/// Based on Dijkstra's solution, uses binary semaphores so a philosopher knows
/// whether his neighbours are currently eating, and will only attempt to pick
/// up the forks when both neighbours are eating.
pub fn main(tx: Sender<StateMsg>, killswitch: Arc<AtomicBool>, random: bool) {
    //TODO
}
