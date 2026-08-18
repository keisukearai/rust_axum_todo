use std::sync::{Arc, Mutex};

use crate::todo::Todo;

#[derive(Default)]
pub struct AppState {
    pub todos: Mutex<Vec<Todo>>,
    pub next_id: Mutex<u64>,
}

pub type SharedState = Arc<AppState>;
