use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use crate::StateMsg;

/// Based on some other guy's solution, works by assigning a strict ordering 
/// hierarchy to the forks. Philosophers will pick up the lowest fork first.
pub fn main(tx: Sender<StateMsg>, killswitch: Arc<AtomicBool>) {
    //TODO
}
