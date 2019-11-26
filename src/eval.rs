use crate::tokens::*;
use crate::parser::*;
use rustyline::Editor;

pub struct LispEnv {
    pub ctx: LispContext<LispToken>,
}

impl LispEnv {
    pub fn repl(&mut self) {
        let mut editor = Editor::<()>::new();

        println!("");

        'repl: loop {  
            if let Ok(mut line) = editor.readline("* ") {
                line.push(' ');

                match parse(&line.chars().collect()) {
                    Ok(xs) => {
                        match &mut self.eval(&xs) {
                            Err(err) => {
                                if let LispError::Quit = err {
                                    println!("");
                                    break 'repl;
                                } else {
                                    editor.add_history_entry(line.as_str());
                                    println!("{}", err);
                                }
                            },
                            res => {
                                editor.add_history_entry(line.as_str());
                                LispToken::display_result(res);
                            }
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            } else {
                break;
            }
        }

        editor.save_history("./session.lisp").unwrap();
    }
    
    fn eval(&mut self, expr: &LispToken) -> Result<LispToken, LispError> {
        eval(&mut self.ctx, expr)
    }
}

impl Default for LispEnv {
    fn default() -> Self {
        let mut default_ctx = LispContext::new();
        
        default_ctx.insert(&String::from("#t"), LispToken::from(true));
        default_ctx.insert(&String::from("#f"), LispToken::from(false));

        default_ctx.insert(&String::from("+"), LispToken::Func(add));
        default_ctx.insert(&String::from("-"), LispToken::Func(sub));
        default_ctx.insert(&String::from("*"), LispToken::Func(mul));
        default_ctx.insert(&String::from("/"), LispToken::Func(div));

        default_ctx.insert(&String::from(">"), LispToken::Func(gt));
        default_ctx.insert(&String::from("<"), LispToken::Func(lt));

        default_ctx.insert(&String::from("and"), LispToken::Func(and));
        default_ctx.insert(&String::from("or"), LispToken::Func(or));
        default_ctx.insert(&String::from("not"), LispToken::Func(not));

        default_ctx.insert(&String::from("cons"), LispToken::Func(cons));
        default_ctx.insert(&String::from("car"), LispToken::Func(car));
        default_ctx.insert(&String::from("cdr"), LispToken::Func(cdr));

        default_ctx.insert(&String::from("eq"), LispToken::Func(eq));
        default_ctx.insert(&String::from("neq"), LispToken::Func(neq));

        default_ctx.insert(&String::from("atom"), LispToken::Func(atom));
        default_ctx.insert(&String::from("cond"), LispToken::Func(cond));
        default_ctx.insert(&String::from("quote"), LispToken::Func(quote));

        default_ctx.insert(&String::from("let"), LispToken::Func(label));
        default_ctx.insert(&String::from("lambda"), LispToken::Func(lambda));
        default_ctx.insert(&String::from("apply"), LispToken::Func(apply));
        
        default_ctx.insert(&String::from("quit"), LispToken::Func(quit));
        
        LispEnv {
            ctx: default_ctx
        }
    }
}

fn eval(ctx: &mut LispContext<LispToken>, expr: &LispToken) -> Result<LispToken, LispError> {
    match expr {
        LispToken::List(lst) => {
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

                if let Ok(LispToken::Func(func)) = symbol {
                    let v = lst.iter().skip(1).map(|tok| tok.clone()).collect();
                    return func(ctx, &v);
                } else if let Err(err) = symbol {
                    return Err(err);
                }
                
                let new = LispToken::List(lst.iter().map(|tok| eval(ctx, tok).unwrap()).collect());

                if format!("{}", new).contains("lambda") {
                    return eval(ctx, &new);
                } else {
                    return Ok(new);
                }

            } else {
                Ok(LispToken::List(lst.iter().map(|tok| eval(ctx, tok).unwrap()).collect()))
            }
        },
        LispToken::Sym(s) => {
            if let Some(sym) = ctx.get(s.to_string()) {
                Ok((*sym).clone())
            } else {
                Err(LispError::EvalError(format!("undefined symbol {:?}", expr.clone())))
            }
        },
        LispToken::Num(_) => {
            Ok(expr.clone())
        },
        LispToken::Str(_) => {
            Ok(expr.clone())
        },
        _ => {
            Err(LispError::EvalError("unexpected expression.".to_string()))
        },
    }
}

fn add(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 2 {
        Err(LispError::EvalError("insufficent number of arguments given".to_string()))
    } else {
        let mut result : f32 = 0.0;

        for arg in args.iter().map(|tok| eval(ctx, tok).unwrap().to_float()) {
            match arg {
                Ok(value) => result = result + value,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(LispToken::from(result))
    }
}

fn sub(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 2 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let mut result : f32 = match eval(ctx, &args[0].clone()).unwrap().to_float() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        for arg in args.iter().skip(1).map(|tok| eval(ctx, tok).unwrap().to_float()) {
            match arg {
                Ok(value) => result = result - value,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(LispToken::from(result))
    }
}

fn mul(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() > 2 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let mut result : f32 = match eval(ctx, &args[0].clone()).unwrap().to_float() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        for arg in args.iter().skip(1).map(|tok| eval(ctx, tok).unwrap().to_float()) {
            match arg {
                Ok(value) => result = result * value,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(LispToken::from(result))
    }
}

fn div(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 2 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let mut result : f32 = match eval(ctx, &args[0].clone()).unwrap().to_float() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        for arg in args.iter().skip(1).map(|tok| eval(ctx, tok).unwrap().to_float()) {
            match arg {
                Ok(value) => result = result / value,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(LispToken::from(result))
    }
}

fn lt(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
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
                let (x, y) : (f32, f32) = (a.parse().unwrap(), b.parse().unwrap());

                if x < y {
                    Ok(LispToken::Sym("#t".to_string()))
                } else {
                    Ok(LispToken::Sym("#f".to_string()))
                }
            },
            _ => Err(LispError::EvalError("invalid arguments given.".to_string()))
        }
    }
}

fn gt(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
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
                let (x, y) : (f32, f32) = (a.parse().unwrap(), b.parse().unwrap());

                if x > y {
                    Ok(LispToken::Sym("#t".to_string()))
                } else {
                    Ok(LispToken::Sym("#f".to_string()))
                }
            },
            _ => Err(LispError::EvalError("invalid arguments given.".to_string()))
        }
    }
}

fn and(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 2 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let mut result : bool = match eval(ctx, &args[0].clone()).unwrap().to_bool() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        for arg in args.iter().skip(1).map(|tok| eval(ctx, tok).unwrap().to_bool()) {
            match arg {
                Ok(value) => result = result && value,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(LispToken::from(result))
    }
}

fn or(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 2 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let mut result : bool = match eval(ctx, &args[0].clone()).unwrap().to_bool() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        for arg in args.iter().skip(1).map(|tok| eval(ctx, tok).unwrap().to_bool()) {
            match arg {
                Ok(value) => result = result || value,
                Err(err) => {
                    return Err(err);
                }
            }

            if result {
                return Ok(LispToken::from(result));
            }
        }

        Ok(LispToken::from(result))
    }
}

fn not(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 1 {
        Err(LispError::EvalError("insufficent number of arguments given.".to_string()))
    } else {
        let result : bool = match eval(ctx, &args[0].clone()).unwrap().to_bool() {
            Ok(value) => value,
            Err(err) => {
                return Err(err);
            }
        };

        Ok(LispToken::from(!result))
    }
}

fn cons(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 1 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        Ok(LispToken::List(args.clone()))
    }
}

fn car(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 1 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {
            Ok(lst[0].clone())
        } else {
            Ok(LispToken::Sym("nil".to_string()))
        }
    }
}

fn cdr(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 1 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        if let Ok(LispToken::List(lst)) = eval(ctx, &args[0]) {
            Ok(LispToken::List(lst.iter().cloned().skip(1).collect()))
        } else {
            Ok(LispToken::Sym("nil".to_string()))
        }
    }
}


fn atom(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 1 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        match args[0] {
            LispToken::List(_) => Ok(LispToken::Sym("#f".to_string())),
            _ => Ok(LispToken::Sym("#t".to_string()))
        }
    }
}

fn cond(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() < 1 {
        return Err(LispError::EvalError("incorrect number of arguments given.".to_string()));
    } else {
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
    }
    Ok(LispToken::Sym("#nil".to_string()))
}

fn eq(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        if eval(ctx, &args[0]).unwrap() == eval(ctx, &args[1]).unwrap() {
            Ok(LispToken::Sym("#t".to_string()))
        } else {
            Ok(LispToken::Sym("#f".to_string()))
        }
    }
}

fn neq(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        match eq(ctx, args) {
            Ok(res) => not(ctx, &vec![res]),
            Err(err) => Err(err)
        }
    }
}

fn quote(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 1 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        Ok(args[0].clone())
    }
}

fn label(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        Err(LispError::EvalError("incorrect number of arguments given.".to_string()))
    } else {
        if let LispToken::Sym(s) = args[0].clone() {
            match eval(ctx, &args[1]) {
                Ok(tok) => {
                    match eval(ctx, &tok) {
                        Ok(result) => ctx.insert(&s, result),
                        Err(err) => return Err(err)
                    }
                    Ok(tok)
                },
                Err(err) => {
                    Err(err)
                }
            }

        } else {
            Err(LispError::EvalError("invalid token.".to_string()))
        }
    }
}

fn lambda(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
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

fn apply(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    if args.len() != 2 {
        return Err(LispError::EvalError("incorrect number of arguments given.".to_string()));
    } else {
        let lambda = match args[0] {
            LispToken::List(_) => args[0].clone(),
            _ => match ctx.get(format!("{}", args[0])) {
                None => LispToken::List(vec![]),
                Some(f) => f.clone()
            }
        };

        if let LispToken::List(f) = lambda {
            if f.len() != 3 {
                return Err(LispError::EvalError("incorrect number of arguments given.".to_string()));
            }

            if LispToken::Sym("lambda".to_string()) != f[0] {
                return Err(LispError::EvalError("invalid argument given.".to_string()))
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

            let input = match &args[1] {
                LispToken::List(xs) => xs.to_vec(),
                x => vec![x.clone()]
            };

            if input.len() != params.len() {
                return Err(LispError::EvalError("incorrect number of arguments given.".to_string()));
            }

            let mut s = format!("{}", f[2]);

            for (idx, arg) in params.iter().enumerate() {
                s = s.replace(&format!("{}", arg), &format!("{}", input[idx]));
            }

            return match parse(&s.chars().collect()) {
                Ok(xs) => eval(ctx, &xs),
                Err(err) => Err(err)
            };
        }
    }

    Err(LispError::EvalError("execution error".to_string()))
}

fn quit(ctx: &mut LispContext<LispToken>, args: &Vec<LispToken>) -> Result<LispToken, LispError> {
    Err(LispError::Quit)
}