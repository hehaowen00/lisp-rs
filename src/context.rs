use std::collections::{HashMap};

pub struct Context<T> {
    globals: HashMap<String, T>,
    pub locals: HashMap<String, T>
}

impl<T> Context<T> {
    pub fn new() -> Context<T> {
        Context {
            globals: HashMap::new(),
            locals: HashMap::new()
        }
    }

    pub fn get(&self, k: String) -> Option<&T> {
        self.globals.get(&k)
    }

    pub fn insert(&mut self, k: String, s: T) {
        self.globals.insert(k, s);
    }

    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }

    pub fn local_get(&self, k: String) -> Option<&T> {
        self.locals.get(&k)
    }

    pub fn local_insert(&mut self, k: String, s: T) {
        self.locals.insert(k, s);
    }
}