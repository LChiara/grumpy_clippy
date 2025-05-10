use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AppState {
    pub message: String,
}

pub type SharedAppState = Arc<RwLock<AppState>>;

/// Utility to create a new shared app state
pub fn new_shared_state() -> SharedAppState {
    Arc::new(RwLock::new(AppState::default()))
}
