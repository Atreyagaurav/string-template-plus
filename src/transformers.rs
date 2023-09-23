use crate::errors::TransformerError;
use lazy_static::lazy_static;
use regex::Regex;
use titlecase::titlecase;

pub fn apply_tranformers(val: &str, tstr: &str) -> Result<String, TransformerError> {
    let (name, args) = tstr.split_once('(').unwrap();
    let args: Vec<&str> = args.strip_suffix(')').unwrap().split(',').collect();
    match name {
        "f" => float_format(val, args),
        "case" => string_format_case(val, args),
        "calc" => calc(val, args),
        _ => Err(TransformerError::UnknownTranformer(
            name.to_string(),
            val.to_string(),
        )),
    }
}

fn check_arguments_len(f: &'static str, r: usize, g: usize) -> Result<(), TransformerError> {
    if g < r {
        Err(TransformerError::TooFewArguments(f, r, g))
    } else if g > r {
        Err(TransformerError::TooManyArguments(f, r, g))
    } else {
        Ok(())
    }
}

pub fn float_format(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "f";
    check_arguments_len(func_name, 1, args.len())?;
    let format = args[0];
    let val = val
        .parse::<f64>()
        .map_err(|_| TransformerError::InvalidValueType(func_name, "float"))?;
    let mut start = 0usize;
    let mut decimal = 6usize;
    if let Some((d, f)) = format.split_once('.') {
        if !d.is_empty() {
            start = d.parse().map_err(|_| {
                TransformerError::InvalidArgumentType(func_name, d.to_string(), "uint")
            })?;
        }
        if f.is_empty() {
            decimal = 0;
        } else {
            decimal = f.parse().map_err(|_| {
                TransformerError::InvalidArgumentType(func_name, f.to_string(), "uint")
            })?;
        }
    } else {
        if !format.is_empty() {
            decimal = format.parse().map_err(|_| {
                TransformerError::InvalidArgumentType(func_name, format.to_string(), "uint")
            })?;
        }
    }
    Ok(format!("{0:1$.2$}", val, start, decimal))
}

pub fn string_format_case(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "case";
    check_arguments_len(func_name, 1, args.len())?;
    let format = args[0];
    match format.to_lowercase().as_str() {
        "up" => Ok(val.to_uppercase()),
        "down" => Ok(val.to_lowercase()),
        "title" => Ok(titlecase(val)),
        "proper" => Ok({
            let mut c = val.chars();
            match c.next() {
                None => String::new(),
                Some(f) => {
                    f.to_uppercase().collect::<String>() + c.as_str().to_lowercase().as_str()
                }
            }
        }),
        _ => Err(TransformerError::InvalidArgumentType(
            func_name,
            format.to_string(),
            "{up;down;proper;title}",
        )),
    }
}

lazy_static! {
    static ref CALC_NUMBERS: Regex = Regex::new("[0-9.]+").unwrap();
}

pub fn calc(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "calc";
    check_arguments_len(func_name, 1, args.len())?;

    let mut last_match = 0usize;
    let mut result: f64 = val
        .parse()
        .map_err(|_| TransformerError::InvalidValueType(func_name, "float"))?;
    let expr = args[0];
    for cap in CALC_NUMBERS.captures_iter(expr) {
        let m = cap.get(0).unwrap();
        let curr_val = m.as_str().parse().map_err(|_| {
            TransformerError::InvalidArgumentType(func_name, m.as_str().to_string(), "float")
        })?;
        if m.start() == 0 {
            result = curr_val;
        } else {
            match &expr[last_match..m.start()] {
                "+" => result += curr_val,
                "-" => result -= curr_val,
                "/" => result /= curr_val,
                "*" => result *= curr_val,
                "^" => result = result.powf(curr_val),
                s => {
                    return Err(TransformerError::InvalidArgumentType(
                        func_name,
                        s.to_string(),
                        "{+,-,*,/,^}",
                    ))
                }
            };
        }
        last_match = m.end();
    }
    Ok(result.to_string())
}
