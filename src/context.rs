use std::collections::{HashMap};

pub struct Context<T> {
    globals: HashMap<String, T>
}

impl<T> Context<T> {
    pub fn new() -> Context<T> {
        Context {
            globals: HashMap::new()
        }
    }

    pub fn get(&self, k: String) -> Option<&T> {
        self.globals.get(&k)
    }

    pub fn insert(&mut self, k: String, s: T) {
        self.globals.insert(k, s);
    }
}