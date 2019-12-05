use crate::tokens::{LispError, LispToken};

// function: serves to call the actual parsing function.
pub fn parse(expr: &Vec<char>) -> Result<LispToken, LispError> {
    let mut idx = 0;
    parse_rd(&expr, &mut idx)
}

// function: converts a vector of chars to a s-expression. returns LispToken on success or LispError on error.
fn parse_rd(expr: &Vec<char>, idx: &mut usize) -> Result<LispToken, LispError> {
    loop {
        if *idx >= expr.len() {
            return Err(LispError::EndOfSequence);
        }

        let ch = expr[*idx];
        let ahead = if *idx + 1 >= expr.len() {
            ' '
        } else {
            expr[*idx + 1]
        };

        if ch.is_alphabetic() || ch == '#' {
            return symbol(&expr, idx);
        } else if ch.is_numeric() || (ch == '-' && ahead.is_numeric()) {
            return number(&expr, idx);
        } else if ch == '"' {
            return string(&expr, idx);
        } else if ch == '\'' {
            return quote(&expr, idx);
        } else if is_special(ch) {
            return special(&expr, idx);
        } else if ch == '(' {
            return list(&expr, idx);
        } else if !is_delimiter(ch) && !is_special(ch) {
            return Err(LispError::UnexpectedChar(ch, *idx));
        } else {
            *idx = *idx + 1;
        }
    }
}

// function: reads in a sequence of numeric characters or symbols into a buffer and stores it in a Num variant
fn number(expr: &Vec<char>, idx: &mut usize) ->  Result<LispToken, LispError> {
    let mut s = expr[*idx].to_string();
    let mut decimal_set = false;

    loop {
        *idx = *idx + 1;

        if *idx >= expr.len() {
            return Err(LispError::EndOfSequence);
        }

        let ch = expr[*idx];

        if ch.is_numeric() {
            s.push(ch);
        } else if ch == '.' && !decimal_set {
            s.push(ch);
            decimal_set = true;
        } else if is_delimiter(ch) {
            *idx = *idx - 1;
            break;
        } else if !is_bracket(ch) {
            return Err(LispError::UnexpectedChar(ch, *idx));
        }
    }

    Ok(LispToken::Num(s))
}

// function: reads in the next LispToken stores them in a Quote variant using the format! macro.
fn quote(expr: &Vec<char>, idx: &mut usize) ->  Result<LispToken, LispError> {
    *idx = *idx + 1;

    let value = parse_rd(expr, idx)?;
    Ok(LispToken::Quote(format!("{} ", value)))
}

// function: reads in alphanumeric characters and stores them in a Sym variant.
fn symbol(expr: &Vec<char>, idx: &mut usize) ->  Result<LispToken, LispError> {
    let mut s = expr[*idx].to_string();

    loop {
        *idx = *idx + 1;

        if *idx >= expr.len() {
            return Err(LispError::EndOfSequence);
        }

        let ch = expr[*idx];

        if ch.is_alphanumeric() || ch == '-' || ch == '#' {
            s.push(ch);
        } else if is_delimiter(ch) {
            *idx = *idx - 1;
            break;
        } else {
            return Err(LispError::UnexpectedChar(ch, *idx));
        }
    }

    Ok(LispToken::Sym(s))
}

// function: reads in special characters or symbols and stores them in a Sym variant.
fn special(expr: &Vec<char>, idx: &mut usize) -> Result<LispToken, LispError> {
    let mut s = expr[*idx].to_string();
    let mut not_set = true;

    loop {
        *idx = *idx + 1;

        if *idx >= expr.len() {
            return Err(LispError::EndOfSequence);
        }

        let ch = expr[*idx];

        if not_set && is_special(ch) {
            s.push(ch);
            not_set = false;
        } else if is_delimiter(ch) {
            *idx = *idx - 1;
            return Ok(LispToken::Sym(s));
        } else {
            return Err(LispError::UnexpectedChar(ch, *idx));
        }
    }
}

// function: reads in a sequence of characters, starting and ending with " and stores them in a Str variant.
fn string(expr: &Vec<char>, idx: &mut usize) -> Result<LispToken, LispError> {
    let mut s = expr[*idx].to_string();

    loop {
        *idx = *idx + 1;

        if *idx >= expr.len() {
            return Err(LispError::EndOfSequence);
        }

        let ch = expr[*idx];
        s.push(ch);

        if ch == '"' {
            return Ok(LispToken::Str(s));
        }
    }
}

// function: stores LispTokens from parse_rd in a vector and stores them in a List variant.
fn list(expr: &Vec<char>, idx: &mut usize) ->  Result<LispToken, LispError> {
    let mut lst = Vec::new();

    loop {
        *idx = *idx + 1;

        if *idx >= expr.len() {
            return Err(LispError::Other("expected closing ')".to_string()));
        }

        if expr[*idx] == ')' {
            break;
        }

        let token = parse_rd(&expr, idx)?;
        lst.push(token);
    }

    Ok(LispToken::List(lst))
}

// Helper Functions

fn is_bracket(ch: char) -> bool {
    ch == '(' || ch == ')'
}

fn is_special(ch: char) -> bool {
    "+-*/%<>".contains(ch)
}

fn is_delimiter(ch: char) -> bool {
    is_bracket(ch) || ch.is_whitespace()
}
