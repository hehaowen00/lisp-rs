extern crate itertools;
extern crate rustyline;

mod tokens;
mod parser;
mod eval;

use eval::{LispEnv};

fn main() {
    let mut env = LispEnv::default();
    env.repl();
}