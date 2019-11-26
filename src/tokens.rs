use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use itertools::Itertools;

pub struct LispContext<T> {
    globals: HashMap<String, T>
}

impl<T> LispContext<T> {
    pub fn new() -> LispContext<T> {
        LispContext {
            globals: HashMap::new()
        }
    }

    pub fn get(&self, k: String) -> Option<&T> {
        self.globals.get(&k)
    }

    pub fn insert(&mut self, k: &String, s: T) {
        self.globals.insert(k.to_string(), s);
    }
}

#[derive(Debug)]
pub enum LispError {
    EndOfSequence,
    EvalError(String),
    Quit,
    NotImplemented,
    Other(String),
    UnexpectedChar(char, usize),
}

impl fmt::Display for LispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LispError::EndOfSequence => {
                write!(f, "error: reached of sequence.")
            },
            LispError::EvalError(msg) => {
                write!(f, "error: {}", msg)
            },
            LispError::NotImplemented => {
                write!(f, "error: not implemented.")
            },
            LispError::UnexpectedChar(ch, idx) => {
                write!(f, "error: unexpected character '{}' at col {}.", ch, idx)
            },
            LispError::Other(msg) => {
                write!(f, "error: {}.", msg)
            },
            _ => write!(f, "")
        }
    }
}

#[derive(Clone)]
pub enum LispToken {
    Func(fn(&mut LispContext<Self>, &Vec<Self>) -> Result<Self, LispError>),
    List(Vec<Self>),
    Num(String),
    Str(String),
    Sym(String)
}

impl LispToken {
    pub fn display_result(result: &Result<LispToken, LispError>) {
        match result {
            Ok(res) => println!("{}\n", res),
            Err(err) => println!("{}\n", err)
        }
    }

    pub fn to_float(&self) -> Result<f32, LispError> {
        match self {
            LispToken::Num(s) => Ok(s.parse().unwrap()),
            _ => Err(LispError::EvalError(format!("value is not a number `{}`", self)))
        }
    }

    pub fn to_bool(&self) -> Result<bool, LispError> {
        if let LispToken::Sym(s) = self {
            if s == "#t" {
                return Ok(true);
            } else if s == "#f" || s == "#nil" {
                return Ok(false);
            }
        }
        return Err(LispError::EvalError("value is not a boolean.".to_string()));
    }
}

impl From<f32> for LispToken {
    fn from(num: f32) -> Self {
        LispToken::Num(format!("{}", num))
    }
}

impl From<bool> for LispToken {
    fn from(value: bool) -> Self {
        if value {
            LispToken::Sym("#t".to_string())
        } else {
            LispToken::Sym("#f".to_string())
        }
    }
}

impl fmt::Debug for LispToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            LispToken::Func(_) => {
                write!(f, "Fn<()>")
            },
            LispToken::List(lst) => {
                let xs = lst.iter().map(|v| format!("{:?}", v)).join(" ");
                if xs.is_empty() {
                    write!(f, "List([])")
                } else {
                    write!(f, "List([ {} ])", xs)
                }
            },
            LispToken::Num(n) => {
                write!(f, "Num({})", n)
            },
            LispToken::Str(string) => {
                write!(f, "Str({})", string)
            },
            LispToken::Sym(string) => {
                write!(f, "Sym(\"{}\")", string)
            }
        }
    }
}

impl fmt::Display for LispToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            LispToken::Func(_) => {
                write!(f, "Fn<()>")
            },
            LispToken::List(lst) => {
                let xs = lst.iter().map(|v| format!("{}", v)).join(" ");
                if xs.is_empty() {
                    write!(f, "()")
                } else {
                    write!(f, "({})", xs)
                }
            },
            LispToken::Num(n) => {
                write!(f, "{}", n)
            },
            LispToken::Str(string) => {
                write!(f, "{}", string)
            },
            LispToken::Sym(string) => {
                write!(f, "{}", string)
            }
        }
    }
}

impl PartialEq for LispToken {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LispToken::Num(a), LispToken::Num(b)) => { a == b },
            (LispToken::Str(a), LispToken::Str(b)) => { a == b },
            (LispToken::Sym(a), LispToken::Sym(b)) => { a == b },
            _ => false
        }
    }
}