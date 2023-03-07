use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use crate::StateMsg;

/// Essentially a 'control' to compare other solutions against. Purely 
/// sequential implementation with no concurrency. 
pub fn main(tx: Sender<StateMsg>, killswitch: Arc<AtomicBool>) {
    //TODO
}
