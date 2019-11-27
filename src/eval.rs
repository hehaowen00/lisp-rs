use crate::context::{LispContext};
use crate::tokens::{LispError, LispToken};
use crate::parser::{parse};
use rustyline::{Editor};

type LispResult = Result<LispToken, LispError>;

pub struct LispEnv {
    ctx: LispContext
}

impl LispEnv {
    pub fn repl(&mut self) {
        let mut editor = Editor::<()>::new();

        'repl: loop {  
            let read_result = editor.readline("* ");
            if let Err(_) = read_result {
                break 'repl;
            }

            let mut line = read_result.unwrap();
            line = line.trim_end().to_string();
            line.push(' ');

            let result = parse(&line.chars().collect());

            if let Err(err) = result {
                println!("{}", err);
                continue;
            }

            match &mut self.eval(&result.unwrap()) {
                Ok(res) => {
                    editor.add_history_entry(line.trim_end());
                    println!("> {}\n", res);
                },
                Err(err) => {
                    if let LispError::Quit = err {
                        println!("");
                        break 'repl;
                    }

                    editor.add_history_entry(line.trim_end());
                    println!("{}\n", err);
                }
            }

            self.ctx.clear_locals();
        }

        editor.save_history("./session.lisp").unwrap();
    }
    
    fn eval(&mut self, expr: &LispToken) -> LispResult {
        eval(&mut self.ctx, expr)
    }
}

impl Default for LispEnv {
    fn default() -> Self {
        let mut symbols = LispContext::new();
        
        symbols.insert(String::from("#t"), LispToken::from(true));
        symbols.insert(String::from("#f"), LispToken::from(false));

        symbols.insert(String::from("+"), LispToken::Func(add));
        symbols.insert(String::from("-"), LispToken::Func(sub));
        symbols.insert(String::from("*"), LispToken::Func(mul));
        symbols.insert(String::from("/"), LispToken::Func(div));

        symbols.insert(String::from(">"), LispToken::Func(gt));
        symbols.insert(String::from("<"), LispToken::Func(lt));

        symbols.insert(String::from("and"), LispToken::Func(and));
        symbols.insert(String::from("or"), LispToken::Func(or));
        symbols.insert(String::from("not"), LispToken::Func(not));

        symbols.insert(String::from("cons"), LispToken::Func(cons));
        symbols.insert(String::from("car"), LispToken::Func(car));
        symbols.insert(String::from("cdr"), LispToken::Func(cdr));

        symbols.insert(String::from("eq"), LispToken::Func(eq));
        symbols.insert(String::from("neq"), LispToken::Func(neq));

        symbols.insert(String::from("atom"), LispToken::Func(atom));
        symbols.insert(String::from("cond"), LispToken::Func(cond));
        symbols.insert(String::from("quote"), LispToken::Func(quote));

        symbols.insert(String::from("let"), LispToken::Func(label));
        symbols.insert(String::from("lambda"), LispToken::Func(lambda));
        symbols.insert(String::from("apply"), LispToken::Func(apply));
        symbols.insert(String::from("quit"), LispToken::Func(quit));
        
        LispEnv {
            ctx: symbols
        }
    }
}

fn eval(ctx: &mut LispContext, expr: &LispToken) -> LispResult {
    match expr {
        LispToken::List(_) => {
            eval_list(ctx, expr)
        },
        LispToken::Sym(s) => {
            if let Some(sym) = ctx.get(s.to_string()) {
                return Ok((*sym).clone());
            }

            Err(LispError::EvalError(format!("undefined symbol `{:?}`", expr.clone())))
        },
        LispToken::Num(_) => {
            Ok(expr.clone())
        },
        LispToken::Str(_) => {
            Ok(expr.clone())
        },
        _ => {
            Err(LispError::EvalError("unexpected expression.".to_string()))
        }
    }
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
        let symbol = eval(ctx, &sym.clone());

        if let Err(_) = symbol {
            return symbol;
        }

        if let LispToken::Func(func) = symbol.unwrap() {
            let v = lst.iter().skip(1).map(|tok| tok.clone()).collect();
            return func(ctx, &v);
        }

        // For all other tokens that aren't Func
        
        let mut xs = Vec::new();

        for item in lst.iter() {
            let result = eval(ctx, item);

            if let Err(_) = result {
                return result;
            }

            xs.push(result.unwrap());
        }

        let token_xs = LispToken::List(xs);

        if format!("{}", token_xs).contains("lambda") {
            return eval(ctx, &token_xs);
        }

        return Ok(token_xs);
    }

    Ok(LispToken::List(lst.iter().map(|tok| eval(ctx, tok).unwrap()).collect()))
}

fn eval_vec(ctx: &mut LispContext, args: &Vec<LispToken>) -> Result<Vec<LispToken>, LispError> {
    let mut xs = Vec::new();

    for arg in args {
        match eval(ctx, arg) {
            Ok(x) => xs.push(x),
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(xs)
}

fn add(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_float(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            let result : f64 = xs.iter().sum();
            Ok(LispToken::from(result))
        },
        Err(err) => {
            Err(err)
        }
    }
}

fn sub(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_float(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            let value : f64 = xs.iter().skip(1).sum();
            Ok(LispToken::from(xs[0] - value))
        },
        Err(err) => {
            return Err(err);
        }
    }
}

fn mul(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_float(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            let mut result : f64 = xs[0];
            for value in xs.iter().skip(1) {
                result = result * value;
            }
            
            Ok(LispToken::from(result))
        },
        Err(err) => {
            return Err(err);
        }
    }
}

fn div(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_float(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            let mut result : f64 = xs[0];
            for value in xs.iter().skip(1) {
                result = result / value;
            }

            Ok(LispToken::from(result))
        },
        Err(err) => {
            return Err(err);
        }
    }
}

fn lt(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } 

    let a = match eval(ctx, &args[0].clone()) {
        Ok(value) => value,
        Err(err) => {
            return Err(err);
        }
    };

    let b = match eval(ctx, &args[1].clone()) {
        Ok(value) => value,
        Err(err) => {
            return Err(err);
        }
    };

    match (a, b) {
        (LispToken::Num(a), LispToken::Num(b)) => {
            let (x, y) : (f64, f64) = (a.parse().unwrap(), b.parse().unwrap());

            if x < y {
                Ok(LispToken::from(true))
            } else {
                Ok(LispToken::from(false))
            }
        },
        _ => Err(LispError::InvalidArgument)
    }
}

fn gt(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let a = match eval(ctx, &args[0].clone()) {
        Ok(value) => value,
        Err(err) => {
            return Err(err);
        }
    };

    let b = match eval(ctx, &args[1].clone()) {
        Ok(value) => value,
        Err(err) => {
            return Err(err);
        }
    };

    match (a, b) {
        (LispToken::Num(a), LispToken::Num(b)) => {
            let (x, y) : (f64, f64) = (a.parse().unwrap(), b.parse().unwrap());

            if x > y {
                Ok(LispToken::from(true))
            } else {
                Ok(LispToken::from(false))
            }
        },
        _ => Err(LispError::InvalidArgument)
    }
}

fn and(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_bool(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            let mut result : bool = xs[0];
            for value in xs.iter().skip(1) {
                result = result && *value;
            }

            Ok(LispToken::from(result))
        },
        Err(err) => {
            return Err(err);
        }
    }
}

fn or(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    match LispToken::to_vec_bool(&lst) {
        Ok(xs) => {
            if xs.len() < 2 {
                return Err(LispError::InvalidNoArguments);
            }

            Ok(LispToken::from(xs.contains(&true)))
        },
        Err(err) => {
            return Err(err);
        }
    }
}

fn not(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    let result : bool = match eval(ctx, &args[0].clone()).unwrap().to_bool() {
        Ok(value) => value,
        Err(err) => {
            return Err(err);
        }
    };

    Ok(LispToken::from(!result))
}

fn cons(_ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 1 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::List(args.clone()))
}

fn car(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {
        if lst.is_empty() {
            return Ok(LispToken::Sym("nil".to_string()));
        }

        return Ok(lst[0].clone());
    }

    Ok(LispToken::Sym("nil".to_string()))
}

fn cdr(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {
        return Ok(LispToken::List(lst.iter().cloned().skip(1).collect()));
    }

    Ok(LispToken::Sym("nil".to_string()))
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

            match eval(ctx, &lst[0]) {
                Ok(result) => {
                    if let Ok(true) = result.to_bool() {
                        return eval(ctx, &lst[1]);
                    }
                },
                Err(err) => {
                    return Err(err);   
                }
            }
        }
    }

    Ok(LispToken::Sym("#nil".to_string()))
}

fn eq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = match eval_vec(ctx, args) {
        Ok(xs) => xs,
        Err(err) => {
            return Err(err);
        }
    };

    if lst.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(lst[0] == lst[1]))
}

fn neq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } 

    match eq(ctx, args) {
        Ok(res) => not(ctx, &vec![res]),
        Err(err) => Err(err)
    }
}

fn quote(_ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(args[0].clone())
}

fn label(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } 

    if let LispToken::Sym(s) = args[0].clone() {
        match eval(ctx, &args[1]) {
            Ok(tok) => {
                match eval(ctx, &tok) {
                    Ok(result) => ctx.insert(s, result),
                    Err(err) => return Err(err)
                }
                Ok(tok)
            },
            Err(err) => {
                Err(err)
            }
        }

    } else {
        Err(LispError::InvalidArgument)
    }
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

fn apply(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } else {
        let symbol = match args[0] {
            LispToken::List(_) => Some(args[0].clone()),
            _ => match ctx.get(format!("{}", args[0])) {
                Some(f) => Some(f.clone()),
                None => None
            }
        };

        let arguments = match &args[1] {
            LispToken::List(xs) => xs.to_vec(),
            x => vec![x.clone()]
        };

        if let Some(LispToken::List(f)) = symbol {
            if f.len() != 3 {
                return Err(LispError::InvalidNoArguments);
            }

            if LispToken::Sym("lambda".to_string()) != f[0] {
                return Err(LispError::InvalidArgument)
            }

            let params : Vec<LispToken> = match &f[1] {
                LispToken::List(xs) => xs.to_vec(),
                _ => vec![]
            };
            
            let expr = match &f[2] {
                LispToken::List(xs) => xs.to_vec(),
                _ => vec![]
            };

            if expr.len() == 0 {
                return Ok(LispToken::Sym("nil".to_string()));
            }

            if arguments.len() != params.len() {
                return Err(LispError::InvalidNoArguments);
            }

            let mut s = format!("{}", f[2]);

            for (idx, arg) in params.iter().enumerate() {
                s = s.replace(&format!("{}", arg), &format!("{}", arguments[idx]));
            }

            return match parse(&s.chars().collect()) {
                Ok(xs) => {
                    match ctx.get_local(format!("{}", &xs)) {
                        Some(result) => Ok(result.clone()),
                        None => {
                            let res = eval(ctx, &xs);
                            if let Ok(value) = &res {
                                ctx.insert_local(format!("{}", &xs), value.clone());
                            }

                            res
                        }
                    }
                },
                Err(err) => Err(err)
            };
        }

        if let Some(LispToken::Func(func)) = symbol {
            return func(ctx, &arguments);
        }
        
        return Err(LispError::InvalidArgument);
    }
}

fn quit(_ctx: &mut LispContext, _args: &Vec<LispToken>) -> LispResult {
    Err(LispError::Quit)
}