use crate::context::{LispContext};
use crate::tokens::{LispError, LispToken};
use crate::parser::{parse};

use rustyline::{Editor};
use rustyline::error::ReadlineError;

// type alias: for repeated return types for lisp primitives.
type LispResult = Result<LispToken, LispError>;

// struct: Stores the environment of which user entered lisp code executes.
// ctx: LispContext for storing globally defined symbols and tail call optimisation.
// result: stores the last result computed for display.
// status: determines when to quit the REPL.
pub struct LispEnv {
    ctx: LispContext,
    result: String,
    status: bool
}

impl LispEnv {

    // function: starts the read-eval-print loop
    pub fn repl(&mut self) {
        let mut editor = Editor::<()>::new();
        let _ = editor.load_history("./session.lisp");

        'repl: loop {
            let line = match editor.readline("* ") {
                Ok(s) => format!("{} ", s),
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(_) => {
                    println!(" > error: unable to read line.\n");
                    continue;
                }
            };

            if line.is_empty() {
                println!("");
                continue;
            }

            match parse(&line.chars().collect()) {
                Ok(expr) => self.eval(&expr),
                Err(err) => self.result = format!("{}", err)
            }

            if !self.status {
                break 'repl;
            }

            println!(" > {}\n", self.result);

            editor.add_history_entry(line.trim_end());
            self.ctx.clear_locals();
        }

        editor.save_history("./session.lisp").unwrap();
    }

    // function: evaluates user lisp expression and stores the result in self
    fn eval(&mut self, expr: &LispToken) {
        match eval(&mut self.ctx, expr) {
            Ok(res) => self.result = format!("{}", res),
            Err(err) => {
                self.status = !(err == LispError::Quit);
                self.result = format!("{}", err)
            }
        }
    }
}

impl Default for LispEnv {
    fn default() -> Self {
        let mut symbols = LispContext::new();

        symbols.insert("#t", LispToken::from(true));
        symbols.insert("#f", LispToken::from(false));
        symbols.insert("#nil", LispToken::from(false));

        symbols.insert("+", LispToken::Func(add));
        symbols.insert("-", LispToken::Func(sub));
        symbols.insert("*", LispToken::Func(mul));
        symbols.insert("/", LispToken::Func(div));
        symbols.insert("mod", LispToken::Func(modulo));

        symbols.insert(">", LispToken::Func(gt));
        symbols.insert("<", LispToken::Func(lt));

        symbols.insert("and", LispToken::Func(and));
        symbols.insert("or", LispToken::Func(or));
        symbols.insert("not", LispToken::Func(not));

        symbols.insert("cons", LispToken::Func(cons));
        symbols.insert("car", LispToken::Func(car));
        symbols.insert("cdr", LispToken::Func(cdr));

        symbols.insert("eq", LispToken::Func(eq));
        symbols.insert("neq", LispToken::Func(neq));

        symbols.insert("atom", LispToken::Func(atom));
        symbols.insert("cond", LispToken::Func(cond));
        symbols.insert("quote", LispToken::Func(quote));

        symbols.insert("let", LispToken::Func(label));
        symbols.insert("lambda", LispToken::Func(lambda));
        symbols.insert("apply", LispToken::Func(apply));
        symbols.insert("eval", LispToken::Func(
            |ctx: &mut LispContext, args: &Vec<LispToken>| -> LispResult { eval(ctx, &args[0])}));
        symbols.insert("quit", LispToken::Func(quit));

        LispEnv {
            ctx: symbols,
            result: String::new(),
            status: true
        }
    }
}

fn eval(ctx: &mut LispContext, expr: &LispToken) -> LispResult {
    match ctx.clone().get_local(format!("{}", expr)) {
        Some(r) => return Ok(r.clone()),
        _ => ()
    }

    let result = match expr {
        LispToken::List(_) => {
            eval_list(ctx, expr)
        },
        LispToken::Sym(s) => {
            if let Some(sym) = ctx.clone().get(s.to_string()) {
                if let LispToken::List(_) = &sym {
                    return eval(ctx, sym);
                }
                return Ok(sym.clone());
            }

            Err(LispError::EvalError(format!("undefined symbol `{:?}`", expr.clone())))
        },
        LispToken::Num(_) => {
            Ok(expr.clone())
        },
        LispToken::Quote(token) => {
            let tokens = &parse(&token.chars().collect())?;
            eval(ctx, tokens)
        },
        LispToken::Str(_) => {
            Ok(expr.clone())
        },
        _ => {
            Err(LispError::EvalError(format!("unexpected expression [{}]", expr)))
        }
    }?;

    ctx.insert_local(format!("{}", expr), result.clone());
    Ok(result)
}

fn eval_list(ctx: &mut LispContext, expr: &LispToken) -> LispResult {
    let mut lst = &Vec::new();

    match expr {
        LispToken::List(xs) => lst = xs,
        _ => {}
    }

    if lst.is_empty() {
        return Ok(expr.clone());
    }

    if let LispToken::List(test) = lst[0].clone() {
        if test[0] == LispToken::Sym("lambda".to_string()) {
            return lambda(ctx, &lst);
        }
    }

    if let Some(sym) = lst.iter().next() {
        let symbol = eval(ctx, &sym.clone())?;

        if let LispToken::Func(func) = symbol {
            let v = lst.iter().skip(1).map(|tok| tok.clone()).collect();
            return func(ctx, &v);
        }

        // For all other tokens that aren't Func

        let mut xs = Vec::new();

        for item in lst.iter() {
            let result = eval(ctx, item)?;
            xs.push(result);
        }

        let token_xs = LispToken::List(xs);

        if format!("{}", token_xs).contains("lambda") {
            return eval(ctx, &token_xs);
        }

        return Ok(token_xs);
    }

    let result = eval_vec(ctx, &lst)?;
    Ok(LispToken::List(result))
}

fn eval_vec(ctx: &mut LispContext, args: &Vec<LispToken>) -> Result<Vec<LispToken>, LispError> {
    let mut xs : Vec<LispToken> = Vec::new();

    for arg in args {
        match arg {
            LispToken::Quote(_) => xs.push(arg.clone()),
            x => {
                let value = eval(ctx, x)?;
                xs.push(value.clone());
            }
        }
    }

    Ok(xs)
}

fn add(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let result : f64 = xs.iter().sum();
    Ok(LispToken::from(result))
}

fn sub(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let value : f64 = xs.iter().skip(1).sum();
    Ok(LispToken::from(xs[0] - value))
}

fn mul(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let mut result : f64 = xs[0];
    for value in xs.iter().skip(1) {
        result = result * value;
    }

    Ok(LispToken::from(result))
}

fn div(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let mut result : f64 = xs[0];
    for value in xs.iter().skip(1) {
        result = result / value;
    }

    Ok(LispToken::from(result))
}

fn modulo(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let mut result : f64 = xs[0];

    match xs.iter().skip(1).next() {
        Some(value) => result = result % value,
        _ => return Err(LispError::InvalidNoArguments)
    }

    Ok(LispToken::from(result))
}

fn lt(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(xs[0] < xs[1]))
}

fn gt(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(xs[0] > xs[1]))
}

fn and(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_bool(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(!xs.contains(&false)))
}

fn or(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_bool(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(xs.contains(&true)))
}

fn not(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_bool(&lst)?;

    if xs.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(!xs[0]))
}

fn cons(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let xs = eval_vec(ctx, args)?;

    if xs.len() < 1 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::List(xs))
}

fn car(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    match eval(ctx, &args[0])? {
        LispToken::List(lst) => {
            if lst.is_empty() {
                return Ok(LispToken::Sym("#nil".to_string()));
            }

            let lst = eval_vec(ctx, &lst)?;
            Ok(lst[0].clone())
        },
        _ => Ok(LispToken::Sym("#nil".to_string()))
    }
}

fn cdr(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    match eval(ctx, &args[0])? {
        LispToken::List(lst) => {
            if lst.len() < 1 {
                return Ok(LispToken::Sym("#nil".to_string()));
            }

            let lst = eval_vec(ctx, &lst)?;
            Ok(LispToken::List(lst.iter().cloned().skip(1).collect()))
        },
        _ => Ok(LispToken::Sym("#nil".to_string()))
    }
}

fn atom(_ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    match args[0] {
        LispToken::List(_) => Ok(LispToken::from(false)),
        _ => Ok(LispToken::from(true))
    }
}

fn cond(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 1 {
        return Err(LispError::InvalidNoArguments);
    }

    for arg in args {
        if let LispToken::List(lst) = &arg {
            if lst.len() != 2 {
                return Err(LispError::EvalError("malformed expression.".to_string()));
            }

            let temp = eval(ctx, &lst[0])?;
            if temp.to_bool()? {
                return eval(ctx, &lst[1]);
            }
        }
    }

    Ok(LispToken::Sym("#nil".to_string()))
}

fn eq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let lst = eval_vec(ctx, args)?;

    if lst.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(lst[0] == lst[1]))
}

fn neq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    let temp = eq(ctx, args)?;
    not(ctx, &vec![temp])
}

fn quote(_ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::Quote(format!("'{}", args[0])))
}

fn label(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    if let LispToken::Sym(s) = &args[0] {
        if format!("{:?}", &args[1]).contains("Quote") {
            let value = parse(&format!("{}", &args[1]).chars().collect())?;
            ctx.insert(s.to_string(), value.clone());
            return Ok(value);
        }

        let value = eval(ctx, &args[1])?;
        let result = eval(ctx, &value)?;

        ctx.insert(s.to_string(), result);
        return Ok(value);
    }

    Err(LispError::InvalidArguments)
}

fn lambda(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() == 2 {
        if let LispToken::List(lst) = &args[0] {
            if let LispToken::Sym(s) = &lst[0] {
                if s == "lambda" {
                    return apply(ctx, args);
                }
            }
        }
    }

    let mut lst = vec![LispToken::Sym("lambda".to_string())];
    for token in args {
        lst.push(token.clone());
    }

    Ok(LispToken::List(lst))
}

// function: applies arguments to functions:
fn apply(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    // check if first element is a symbol
    let symbol = match args[0] {
        LispToken::List(_) => Some(args[0].clone()),
        _ => match ctx.get(format!("{}", args[0])) {
            Some(f) => Some(f.clone()),
            None => None
        }
    };

    // check if second element is a list.
    let arguments = match &eval(ctx, &args[1])? {
        LispToken::List(xs) => xs.to_vec(),
        x => vec![x.clone()]
    };

    // check if first element evaluates to a list.
    if let Some(LispToken::List(f)) = symbol {
        if f.len() != 3 {
            return Err(LispError::InvalidNoArguments);
        }

        // checks if list is a lambda function.
        if LispToken::Sym("lambda".to_string()) != f[0] {
            return Err(LispError::InvalidArguments)
        }

        // check parameters to lambda function.
        let params : Vec<LispToken> = match &f[1] {
            LispToken::List(xs) => xs.to_vec(),
            _ => vec![]
        };

        // get the expression to be computed.
        let expr = match &f[2] {
            LispToken::List(xs) => xs.to_vec(),
            _ => vec![]
        };

        // checks the length of the expression to be evaluated.
        if expr.len() == 0 {
            return Ok(LispToken::Sym("#nil".to_string()));
        }

        // checks if the number of input values matches the number of parameters
        if arguments.len() != params.len() {
            return Err(LispError::InvalidNoArguments);
        }

        // convert expression to string using display trait.
        let mut s = format!("{}", f[2]);

        // replace expression parameters with input values.
        for (idx, arg) in params.iter().enumerate() {
            s = s.replace(&format!("{}", arg), &format!("{}", arguments[idx]));
        }

        // turn the string back into a lisp AST.
        let expr = parse(&s.chars().collect())?;

        // evaluate and return the result of the computed lambda.
        return eval(ctx, &expr);
    }

    if let Some(LispToken::Func(func)) = symbol {
        return func(ctx, &arguments);
    }

    return Err(LispError::InvalidArguments);
}

fn quit(_ctx: &mut LispContext, _args: &Vec<LispToken>) -> LispResult {
    Err(LispError::Quit)
}
