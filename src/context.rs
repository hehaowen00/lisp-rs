use crate::tokens::{LispToken};
use std::collections::{HashMap};


// Struct to represent the environment of the Lisp Interpreter
// globals: A hashmap to store globally store defined symbols
// locals: A hashmap to store results of previous computation
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

    pub fn get<T: Into<String>>(&self, k: T) -> Option<&LispToken> {
        self.globals.get(&k.into())
    }

    pub fn insert<T: Into<String>>(&mut self, k: T, v: LispToken) {
        self.globals.insert(k.into(), v);
    }

    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }

    pub fn get_local<T: Into<String>>(&self, k: T) -> Option<&LispToken> {
        self.locals.get(&k.into())
    }

    pub fn insert_local<T: Into<String>>(&mut self, k: T, v: LispToken) {
        self.locals.insert(k.into(), v);
    }
}
