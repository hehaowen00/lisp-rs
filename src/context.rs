use crate::tokens::{LispToken};
use std::collections::{HashMap};

#[derive(Clone)]
pub struct LispContext {
    globals: HashMap<String, LispToken>,
    locals: HashMap<String, LispToken>
}

impl LispContext {
    pub fn new() -> LispContext {
        LispContext {
            globals: HashMap::new(),
            locals: HashMap::new()
        }
    }

    pub fn get(&self, k: String) -> Option<&LispToken> {
        self.globals.get(&k)
    }

    pub fn insert(&mut self, k: String, v: LispToken) {
        self.globals.insert(k, v);
    }

    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }

    pub fn get_local(&self, k: String) -> Option<&LispToken> {
        self.locals.get(&k)
    }

    pub fn insert_local(&mut self, k: String, v: LispToken) {
        self.locals.insert(k, v);
    }
}