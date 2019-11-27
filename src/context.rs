use std::collections::{HashMap};

pub struct Context<T> {
    globals: HashMap<String, T>,
    locals: HashMap<String, T>
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

    pub fn insert(&mut self, k: String, v: T) {
        self.globals.insert(k, v);
    }

    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }

    pub fn get_local(&self, k: String) -> Option<&T> {
        self.locals.get(&k)
    }

    pub fn insert_local(&mut self, k: String, v: T) {
        self.locals.insert(k, v);
    }
}