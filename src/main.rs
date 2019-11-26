extern crate itertools;
extern crate rustyline;

mod context;
mod tokens;
mod parser;
mod eval;

use eval::{LispEnv};

fn main() {
    let mut env = LispEnv::default();
    env.repl();
}