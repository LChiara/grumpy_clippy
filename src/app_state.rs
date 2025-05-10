//! This module defines shared application state management using thread-safe primitives.
//!
//! It utilizes `std::sync::{Arc, RwLock}` to provide a mechanism for safely sharing and
//! modifying state across multiple threads. `Arc` ensures that the state can be shared
//! with reference counting, while `RwLock` allows for concurrent read access and
//! exclusive write access to the state.
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
