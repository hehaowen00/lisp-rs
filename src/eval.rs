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

        let _ = editor.load_history("./session.lisp");

        'repl: loop {  
            let read_result = editor.readline("* ");
            if let Err(_) = read_result {
                break 'repl;
            }

            let mut line = read_result.unwrap();
            line = line.trim_end().to_string();
            line.push(' ');

            match parse(&line.chars().collect()) {
                Ok(expr) => {
                    if !self.eval(&expr) {
                        break 'repl;
                    }
                },
                Err(err) => println!("{}", err)
            }

            editor.add_history_entry(line.trim_end());
            self.ctx.clear_locals();
        }

        editor.save_history("./session.lisp").unwrap();
    }
    
    fn eval(&mut self, expr: &LispToken) -> bool {
        match eval(&mut self.ctx, expr) {
            Ok(res) => println!(" > {}\n", res),
            Err(err) => {
                if let LispError::Quit = err {
                    return false;
                }

                println!("{}\n", err);
            }
        }

        true
    }
}

impl Default for LispEnv {
    fn default() -> Self {
        let mut symbols = LispContext::new();
        
        symbols.insert(String::from("#t"), LispToken::from(true));
        symbols.insert(String::from("#f"), LispToken::from(false));
        symbols.insert(String::from("#nil"), LispToken::from(false));

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
        symbols.insert(String::from("eval"), LispToken::Func(
            |ctx: &mut LispContext, args: &Vec<LispToken>| -> LispResult { eval(ctx, &args[0])}));
        symbols.insert(String::from("quit"), LispToken::Func(quit));
        
        LispEnv {
            ctx: symbols
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
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let result : f64 = xs.iter().sum();
    Ok(LispToken::from(result))
}

fn sub(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_float(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let value : f64 = xs.iter().skip(1).sum();
    Ok(LispToken::from(xs[0] - value))
}

fn mul(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

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
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

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

fn lt(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } 

    let a = eval(ctx, &args[0].clone())?;
    let b = eval(ctx, &args[1].clone())?;

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

    let a = eval(ctx, &args[0].clone())?;
    let b = eval(ctx, &args[1].clone())?;

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

    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_bool(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let mut result : bool = xs[0];
    for value in xs.iter().skip(1) {
        result = result && *value;
    }

    Ok(LispToken::from(result))
}

fn or(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = eval_vec(ctx, args)?;
    let xs = LispToken::to_vec_bool(&lst)?;

    if xs.len() < 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(xs.contains(&true)))
}

fn not(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    let result = eval(ctx, &args[0].clone())?;
    let value : bool = result.to_bool()?; 

    Ok(LispToken::from(!value))
}

fn cons(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() < 1 {
        return Err(LispError::InvalidNoArguments);
    }

    let xs = eval_vec(ctx, args)?;
    Ok(LispToken::List(xs))
}

fn car(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {

        if lst.is_empty() {
            return Ok(LispToken::Sym("#nil".to_string()));
        }

        let lst = eval_vec(ctx, &lst)?;
        return Ok(lst[0].clone());
    }

    Ok(LispToken::Sym("#nil".to_string()))
}

fn cdr(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 1 {
        return Err(LispError::InvalidNoArguments);
    }

    if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {
        let lst = eval_vec(ctx, &lst)?;
        return Ok(LispToken::List(lst.iter().cloned().skip(1).collect()));
    }

    Ok(LispToken::Sym("#nil".to_string()))
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

            let temp1 = eval(ctx, &lst[0])?;
            if temp1.to_bool()? {
                return eval(ctx, &lst[1]);
            }
        }
    }

    Ok(LispToken::Sym("#nil".to_string()))
}

fn eq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    let lst = eval_vec(ctx, args)?;

    if lst.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    }

    Ok(LispToken::from(lst[0] == lst[1]))
}

fn neq(ctx: &mut LispContext, args: &Vec<LispToken>) -> LispResult {
    if args.len() != 2 {
        return Err(LispError::InvalidNoArguments);
    } 

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
    
    Err(LispError::InvalidArgument)
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
    }

    let symbol = match args[0] {
        LispToken::List(_) => Some(args[0].clone()),
        _ => match ctx.get(format!("{}", args[0])) {
            Some(f) => Some(f.clone()),
            None => None
        }
    };

    let arguments = match &eval(ctx, &args[1])? {
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
            return Ok(LispToken::Sym("#nil".to_string()));
        }

        if arguments.len() != params.len() {
            return Err(LispError::InvalidNoArguments);
        }

        let mut s = format!("{}", f[2]);

        for (idx, arg) in params.iter().enumerate() {
            s = s.replace(&format!("{}", arg), &format!("{}", arguments[idx]));
        }

        let expr = parse(&s.chars().collect())?;

        return eval(ctx, &expr);
    }

    if let Some(LispToken::Func(func)) = symbol {
        return func(ctx, &arguments);
    }
    
    return Err(LispError::InvalidArgument);
}

fn quit(_ctx: &mut LispContext, _args: &Vec<LispToken>) -> LispResult {
    Err(LispError::Quit)
}