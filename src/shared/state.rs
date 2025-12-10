use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};

use livesplit_auto_splitting::TimerState;

#[derive(Debug, Clone)]
pub struct SharedState {
    state: Arc<RwLock<TimerState>>,
    alive: Arc<AtomicBool>,
}

impl SharedState {
    pub fn new(state: Arc<RwLock<TimerState>>) -> Self {
        Self {
            state,
            alive: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn state(&self) -> TimerState {
        *self.state.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    pub fn deadge(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }
}
