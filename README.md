# lisp.rs

A lisp interpreter implemented in Rust with basic lookup tail call optimization.
A REPL (read-evaluate-print loop) is also implemented.

Supports boolean, numeric, string and symbol types.

Currently implements the following symbols by default:
```
+ - * / < >

and
or
not

cons
car
cdr

eq
neq

atom
cond
quote

let
lambda
apply
eval
quit
```

## Example
```lisp
(let fact (lambda (x) (cond ((eq x 1) 1) (#t (* x (fact (- x 1)))))))
(fact 10) ; -> 3628800

(let fib (lambda (x) (cond ((< x 2) x) (#t (+ (fib (- x 1)) (fib (- x 2)))))))
(fib 45) ; -> 1134903170
```