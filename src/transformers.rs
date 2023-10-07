/*! Transformers for the template

To apply a tranformer to a variable provide it after [`VAR_TRANSFORM_SEP_CHAR`] (currently ":") to a variable template.

There are a few transformers available:

| Transformer          | Arguments | Function                 | Example                  |
|----------------------|-----------|--------------------------|--------------------------|
| f [`format_float`]   | [.]N      | only N number of decimal | {"1.12":f(.1)} ⇒ 1.1     |
| case [`string_case`] | up        | UPCASE a string          | {"na":case(up)} ⇒ NA     |
| case [`string_case`] | down      | downcase a string        | {"nA":case(down)} ⇒ na   |
| case [`string_case`] | proper    | Upcase the first letter  | {"nA":case(proper)} ⇒ Na |
| case [`string_case`] | title     | Title Case the string    | {"na":case(title)} ⇒ Na  |
| calc                 | [+-*\/^]N  | Airthmatic calculation   | {"1":calc(+1*2^2)} ⇒ 16  |
| calc                 | [+-*\/^]N  | Airthmatic calculation   | {"1":calc(+1,-1)} ⇒ 2,0  |
| count                | str       | count str occurance      | {"nata":count(a)} ⇒ 2    |
| repl [`replace`]     | str1,str2 | replace str1 by str2     | {"nata":rep(a,o)} ⇒ noto |
| q      [`quote`]     | [str1]    | quote with str1, or ""   | {"nata":q()} ⇒ "noto"    |
| take                 | str,N     | take Nth group sep by str| {"nata":take(a,2)} ⇒ "t" |
| trim                 | str       | trim the string with str | {"nata":trim(a)} ⇒ "nat" |

You can chain transformers ones after another for combined actions. For example, `count( ):calc(+1)` will give you total number of words in a sentence.

Examples are in individual functions.
*/
use std::ops::{Bound, RangeBounds};

use crate::errors::TransformerError;
use crate::VAR_TRANSFORM_SEP_CHAR;
use lazy_static::lazy_static;
use regex::Regex;
use titlecase::titlecase;

/// Applies any tranformations to the variable, you can chain the
/// transformers Called whenever you use [`VAR_TRANSFORM_SEP_CHAR`] to
/// provide a transformer in the template.
pub fn apply_tranformers(val: &str, transformations: &str) -> Result<String, TransformerError> {
    let mut val: String = val.to_string();
    for tstr in transformations.split(VAR_TRANSFORM_SEP_CHAR) {
        if tstr.is_empty() {
            continue;
        }
        let (name, args) = tstr.split_once('(').ok_or(TransformerError::InvalidSyntax(
            tstr.to_string(),
            "No opening paranthesis".to_string(),
        ))?;
        let args: Vec<&str> = args
            .strip_suffix(')')
            .ok_or(TransformerError::InvalidSyntax(
                tstr.to_string(),
                "No closing paranthesis".to_string(),
            ))?
            .split(',')
            .collect();
        val = match name {
            "f" => float_format(&val, args)?,
            "case" => string_case(&val, args)?,
            "calc" => calc(&val, args)?,
            "count" => count(&val, args)?,
            "repl" => replace(&val, args)?,
            "take" => take(&val, args)?,
            _ => {
                return Err(TransformerError::UnknownTranformer(
                    name.to_string(),
                    val.to_string(),
                ))
            }
        };
    }
    Ok(val)
}

/// Gets the bound of a rust range object
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// # use std::ops::RangeBounds;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(bound((2..).end_bound(), true), None);
///     assert_eq!(bound((..2).end_bound(), false), Some(1));
///     assert_eq!(bound((..=2).end_bound(), false), Some(2));
///     assert_eq!(bound((..2).start_bound(), true), None);
///     assert_eq!(bound((0..).start_bound(), false), Some(0));
/// # Ok(())
/// # }
pub fn bound(b: Bound<&usize>, lower: bool) -> Option<usize> {
    match b {
        Bound::Unbounded => None,
        Bound::Included(v) => Some(*v),
        Bound::Excluded(v) => Some(if lower { v + 1 } else { v - 1 }),
    }
}

/// Checks whether the arguments lenth matches what is required
fn check_arguments_len<R: RangeBounds<usize>>(
    func_name: &'static str,
    req: R,
    given: usize,
) -> Result<(), TransformerError> {
    if req.contains(&given) {
        Ok(())
    } else {
        match (
            bound(req.start_bound(), true),
            bound(req.end_bound(), false),
        ) {
            (None, Some(r)) => Err(TransformerError::TooManyArguments(func_name, r, given)),
            (Some(r), None) => Err(TransformerError::TooFewArguments(func_name, r, given)),
            (Some(r1), Some(r2)) => {
                if given < r1 {
                    Err(TransformerError::TooFewArguments(func_name, r1, given))
                } else {
                    Err(TransformerError::TooManyArguments(func_name, r2, given))
                }
            }
            _ => Ok(()),
        }
    }
}

/// format the float (numbers). For example with `val=1.123`, `{val:f(2)}` or `{val:f(.2)}` gives `1.12`
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(float_format("1.12", vec![".1"])?, "1.1");
///     assert_eq!(float_format("1.12", vec!["2"])?, "1.12");
///     assert_eq!(float_format("1.12", vec!["0"])?, "1");
/// # Ok(())
/// # }
pub fn float_format(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "f";
    check_arguments_len(func_name, 1..=1, args.len())?;
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

/// Format the string. Supports `up`=> UPCASE, `down`=> downcase, `proper` => first character UPCASE all others downcase, `title` => title case according to [`titlecase::titlecase`]. e.g. `{var:case(up)}`.
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(string_case("na", vec!["up"])?, "NA");
///     assert_eq!(string_case("nA", vec!["down"])?, "na");
///     assert_eq!(string_case("nA", vec!["proper"])?, "Na");
///     assert_eq!(string_case("here, an apple", vec!["title"])?, "Here, an Apple");
/// # Ok(())
/// # }
pub fn string_case(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "case";
    check_arguments_len(func_name, 1..=1, args.len())?;
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

/// Airthmatic calculations, the value needs to be float. e.g. `{val:calc(+1)}` will add 1 to the value. The order of calculation is left to right.
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(calc("1.24", vec!["+1"])?, "2.24");
///     assert_eq!(calc("1", vec!["+1*2^2"])?, "16");
///     assert_eq!(calc("1.24", vec!["+1", "-1"])?, "2.24,0.24");
/// # Ok(())
/// # }
pub fn calc(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "calc";
    check_arguments_len(func_name, 1.., args.len())?;

    let val: f64 = val
        .parse()
        .map_err(|_| TransformerError::InvalidValueType(func_name, "float"))?;
    let mut results: Vec<String> = Vec::new();
    for expr in args {
        let mut last_match = 0usize;
        let mut result = val;
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
        results.push(result.to_string());
    }
    Ok(results.join(","))
}

/// Count the number of occurances of a pattern in the string. You can chain it with [`calc`] to get the number of word like: `{val:count( ):calc(+1)}`
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(count("nata", vec!["a"])?, "2");
///     assert_eq!(count("nata", vec!["a", "t"])?, "2,1");
///     assert_eq!(count("nata", vec![" "])?, "0");
///     assert_eq!(count("hi there fellow", vec![" "])?, "2");
/// # Ok(())
/// # }
pub fn count(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "count";
    check_arguments_len(func_name, 1.., args.len())?;
    let counts: Vec<String> = args
        .iter()
        .map(|sep| val.matches(sep).count().to_string())
        .collect();
    Ok(counts.join(","))
}

/// Replace text in the string, by another text
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(replace("nata", vec!["a", "o"])?, "noto");
///     assert_eq!(replace("hi there fellow", vec![" ", "-"])?, "hi-there-fellow");
/// # Ok(())
/// # }
pub fn replace(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "replace";
    check_arguments_len(func_name, 2..=2, args.len())?;
    Ok(val.replace(args[0], args[1]))
}

/// Split the text with given separator and then take the Nth group
///
/// N=0, will give the whole group separated by comma, but it might
/// give unexpected results if there is already comma in string and
/// you're splitting with something else
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(take("nata", vec!["a", "2"])?, "t");
///     assert_eq!(take("hi there fellow", vec![" ", "2"])?, "there");
///     assert_eq!(take("hi there fellow", vec![" ", "2", "2"])?, "there fellow");
/// # Ok(())
/// # }
pub fn take(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "take";
    check_arguments_len(func_name, 2..=3, args.len())?;
    let n: usize = args[1].parse().map_err(|_| {
        TransformerError::InvalidArgumentType(func_name, args[1].to_string(), "uint")
    })?;
    let spl = if args.len() == 2 {
        val.split(args[0]).nth(n - 1)
    } else {
        val.splitn(
            args[2].parse().map_err(|_| {
                TransformerError::InvalidArgumentType(func_name, args[1].to_string(), "int")
            })?,
            args[0],
        )
        .nth(n - 1)
    };

    Ok(spl.unwrap_or("").to_string())
}

/// Trim the given string with given patterns one after another
///
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(trim("nata", vec!["a"])?, "nat");
///     assert_eq!(trim("  \tnata\t  ", vec![])?, "nata");
///     assert_eq!(trim("hi there! ", vec![" ", "!"])?, "hi there");
///     assert_eq!(trim("hi there! ", vec![" !", "ih"])?, " there");
/// # Ok(())
/// # }
pub fn trim(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "trim";
    check_arguments_len(func_name, .., args.len())?;
    if args.is_empty() {
        return Ok(val.trim().to_string());
    }
    let mut val = val;
    for arg in args {
        val = val.trim_matches(|c| arg.contains(c))
    }

    Ok(val.to_string())
}

/// Quote the text with given strings or `""`
///
/// ```rust
/// # use std::error::Error;
/// # use string_template_plus::transformers::*;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     assert_eq!(quote("nata", vec![])?, "\"nata\"");
///     assert_eq!(quote("nata", vec!["'"])?, "'nata'");
///     assert_eq!(quote("na\"ta", vec![])?, "\"na\\\"ta\"");
///     assert_eq!(quote("na'ta", vec!["'"])?, "'na\\'ta'");
///     assert_eq!(quote("nata", vec!["`", "'"])?, "`nata'");
/// # Ok(())
/// # }
pub fn quote(val: &str, args: Vec<&str>) -> Result<String, TransformerError> {
    let func_name = "quote";
    check_arguments_len(func_name, ..=2, args.len())?;
    Ok(if args.is_empty() {
        format!("{:?}", val)
    } else if args.len() == 1 {
        format!(
            "{0}{1}{0}",
            args[0],
            val.replace(args[0], &format!("\\{}", args[0]))
        )
    } else {
        format!(
            "{}{}{}",
            args[0],
            val.replace(args[0], &format!("\\{}", args[0]))
                .replace(args[1], &format!("\\{}", args[1])),
            args[1]
        )
    })
}
