use anyhow::Context;
use rust_lisp::default_env;
use rust_lisp::interpreter::eval_block;
use rust_lisp::model::{FloatType, RuntimeError, Symbol, Value};
use rust_lisp::parser::{parse, ParseError};
use std::num::ParseFloatError;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// Evaluate the lisp expression
///
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::lisp::*;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let mut vars: HashMap<String, String> = HashMap::new();
///     vars.insert("test".into(), "1".into());
///     assert_eq!(calculate(&vars, "(+ 1 1)")?, "2");
///     assert_eq!(calculate(&vars, "(st+var 'test)")?, "\"1\"");
///     assert_eq!(calculate(&vars, "(/ 20 (st+num \"test\"))")?, "20");
///     assert_eq!(calculate(&vars, "(/ 20 (st+num 'testing 5))")?, "4");
///     assert_eq!(calculate(&vars, "(st+has 'test)")?, "T");
/// # Ok(())
/// # }
pub fn calculate(variables: &HashMap<String, String>, expr: &str) -> anyhow::Result<String> {
    let expr = parse(expr)
        .collect::<Result<Vec<Value>, ParseError>>()
        .ok()
        .context("Parse Failed")?;
    let env = Rc::new(RefCell::new(default_env()));

    // can't figure out how to remove this unnecessary clone
    let vars1 = variables.clone();
    env.borrow_mut().define(
        Symbol::from("st+var"),
        Value::NativeClosure(Rc::new(RefCell::new(move |_, args: Vec<Value>| {
            let name: String = match &args[0] {
                Value::String(s) => s.to_string(),
                Value::Symbol(s) => s.to_string(),
                _ => Err(RuntimeError {
                    msg: "Only Symbol and String can be passed to st+var.".into(),
                })?,
            };
            let val: String = if args.len() == 1 {
                vars1.get(&name).unwrap().into()
            } else if args.len() == 2 {
                vars1
                    .get(&name)
                    .map(|s| s.to_string())
                    .unwrap_or(args[1].to_string())
            } else {
                Err(RuntimeError {
                    msg: "Too many/few arguments in st+var.".into(),
                })?
            };
            Ok(Value::String(val))
        }))),
    );

    let vars2 = variables.clone();
    env.borrow_mut().define(
        Symbol::from("st+num"),
        Value::NativeClosure(Rc::new(RefCell::new(move |_, args: Vec<Value>| {
            let name: String = match &args[0] {
                Value::String(s) => s.to_string(),
                Value::Symbol(s) => s.to_string(),
                _ => Err(RuntimeError {
                    msg: "Only Symbol and String can be passed to st+num.".into(),
                })?,
            };
            let val: String = if args.len() == 1 {
                vars2.get(&name).unwrap().into()
            } else if args.len() == 2 {
                vars2
                    .get(&name)
                    .map(|s| s.to_string())
                    .unwrap_or(args[1].to_string())
            } else {
                Err(RuntimeError {
                    msg: "Too many/few arguments in st+num.".into(),
                })?
            };

            let val: FloatType = val
                .parse()
                .map_err(|e: ParseFloatError| RuntimeError { msg: e.to_string() })?;
            Ok(Value::Float(val))
        }))),
    );

    let vars3: HashSet<String> = variables.iter().map(|(k, _)| k.to_string()).collect();
    env.borrow_mut().define(
        Symbol::from("st+has"),
        Value::NativeClosure(Rc::new(RefCell::new(move |_, args: Vec<Value>| {
            let name: String = match &args[0] {
                Value::String(s) => s.to_string(),
                Value::Symbol(s) => s.to_string(),
                _ => Err(RuntimeError {
                    msg: "Only Symbol and String can be passed to st+num.".into(),
                })?,
            };
            Ok(vars3.get(&name).is_some().into())
        }))),
    );

    // can't define functions it seems, hence the redefinition above
    // env.borrow_mut().define(
    //     Symbol::from("stp-num"),
    //     lisp! {
    //         (lambda (x) ({ Value::Symbol("string-to-number".into())}
    //          ({Value::Symbol("stp-var".into())} x)))
    //     },
    // );

    let res = eval_block(env.clone(), expr.into_iter())?;
    Ok(res.to_string())
}
