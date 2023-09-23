/*!
# Introduction

This is a simple template tool that works with variable names and
[`HashMap`] of [`String`]. The [`Template`] can be parsed from [`str`]
and then you can render it using the variables in [`HashMap`] and any
shell commands running through [`Exec`].

# Features
- Parse the template from a [`str`] that's easy to write,
- Support for alternatives in case some variables are not present,
- Support for literal strings inside the alternative options,
- Support for the date time format using [`chrono`],
- Support for any arbitrary commands, etc.

# Usages
Simple variables:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("hello {name}").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("name".into(), "world".into());
let rendered = templ
.render(&RenderOptions {
variables: vars,
..Default::default()
            })
            .unwrap();
assert_eq!(rendered, "hello world");
# Ok(())
# }
```

Safe replace, blank if not present, or literal string if not present:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("hello {name?} {lastname?\"User\"}").unwrap();
let vars: HashMap<String, String> = HashMap::new();
let rendered = templ
.render(&RenderOptions {
variables: vars,
..Default::default()
            })
            .unwrap();
assert_eq!(rendered, "hello  User");
# Ok(())
# }
```

Alternate, whichever variable it finds first will be replaced:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("hello {nickname?name}").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("name".into(), "world".into());
let rendered = templ
.render(&RenderOptions {
variables: vars,
..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello world");
# Ok(())
# }
```

Custom Commands:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("L=$(printf \"%.2f\" {length})").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("length".into(), "12.342323".into());
let rendered = templ
.render(&RenderOptions {
wd: PathBuf::from("."),
variables: vars,
shell_commands: true,
            })
            .unwrap();
        assert_eq!(rendered, "L=12.34");
# Ok(())
# }
```

You can turn off Custom Commands for safety:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("L=$(printf \"%.2f\" {length})").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("length".into(), "12.342323".into());
let rendered = templ
.render(&RenderOptions {
wd: PathBuf::from("."),
variables: vars,
shell_commands: false,
            })
            .unwrap();
        assert_eq!(rendered, "L=$(printf \"%.2f\" 12.342323)");
# Ok(())
# }
```

Date Time:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use chrono::Local;
# use string_template_plus::{Render, RenderOptions, parse_template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = parse_template("hello {name} at {%Y-%m-%d}").unwrap();
let timefmt = Local::now().format("%Y-%m-%d");
let output = format!("hello world at {}", timefmt);
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("name".into(), "world".into());
let rendered = templ
.render(&RenderOptions {
wd: PathBuf::from("."),
variables: vars,
shell_commands: false,
            })
            .unwrap();
        assert_eq!(rendered, output);
# Ok(())
# }
```

# Limitations
- You cannot use positional arguments in this template system, only named ones. `{}` will be replaced with empty string. Although you can use `"0"`, `"1"`, etc as variable names in the template and the render options variables.
- I haven't tested variety of names, although they should work try to keep the names identifier friendly.
- Currently doesn't have format specifiers, for now you can use the command options with `printf` bash command to format things the way you want.
Like a template `this is $(printf "%.2f" {weight}) kg.` should be rendered with the correct float formatting.
*/
use anyhow::{Context, Error};
use chrono::Local;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use subprocess::Exec;

/// Character to separate the variables. If the first variable is not present it'll use the one behind it and so on. Keep it at the end, if you want a empty string instead of error on missing variable.
pub static OPTIONAL_RENDER_CHAR: char = '?';
/// Character that should be in the beginning of the variable to determine it as datetime format.
pub static TIME_FORMAT_CHAR: char = '%';
/// Quote characters to use to make a value literal instead of a variable. In combination with [`OPTIONAL_RENDER_CHAR`] it can be used as a default value when variable(s) is/are not present.
pub static LITERAL_VALUE_QUOTE_CHAR: char = '"';
static LITERAL_REPLACEMENTS: [&str; 3] = [
    "",  // to replace {} as empty string.
    "{", // to replace {{} as {
    "}", // to replace {}} as }
];

lazy_static! {
    pub static ref VARIABLE_REGEX: Regex = Regex::new(r"\{(.*?)\}").unwrap();
    pub static ref SHELL_COMMAND_REGEX: Regex = Regex::new(r"[$]\((.*?)\)").unwrap();
}

/// Runs a command and returns the output of the command or the error
fn cmd_output(cmd: &str, wd: &PathBuf) -> Result<String, Error> {
    let mut out: String = String::new();
    Exec::shell(cmd)
        .cwd(wd)
        .stream_stdout()?
        .read_to_string(&mut out)?;
    Ok(out)
}

/// Parts that make up a [`Template`]. You can have literal strings, variables, time date format, command, or optional format with [`OPTIONAL_RENDER_CHAR`].
///
/// [`TemplatePart::Lit`] = Literal Strings like `"hi "` in `"hi {name}"`
/// [`TemplatePart::Var`] = Variable part like `"name"` in `"hi {name}"`
/// [`TemplatePart::Time`] = Date time format like `"%F"` in `"Today: {%F}"`
/// [`TemplatePart::Cmd`] = Command like `"echo world"` in `"hello $(echo world)"`
/// [`TemplatePart::Any`] = Optional format like `"name?age"` in `"hello {name?age}"`
///
/// [`TemplatePart::Cmd`] and [`TemplatePart::Any`] can in turn contain other [`TemplatePart`] inside them. Haven't tested on nesting complex ones within each other though.
#[derive(Debug, Clone)]
pub enum TemplatePart {
    Lit(String),
    Var(String),
    Time(String),
    Cmd(Vec<TemplatePart>),
    Any(Vec<TemplatePart>),
}

/// Main Template that get's passed around, consists of `[Vec`] of [`TemplatePart`]
///
/// ```rust
/// # use std::error::Error;
/// # use std::collections::HashMap;
/// # use std::path::PathBuf;
/// # use string_template_plus::{Render, RenderOptions, parse_template};
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let templ = parse_template("hello {nickname?name}. You're $(printf \"%.1f\" {weight})kg").unwrap();
///     let mut vars: HashMap<String, String> = HashMap::new();
///     vars.insert("name".into(), "John".into());
///     vars.insert("weight".into(), "132.3423".into());
///     let rendered = templ
///         .render(&RenderOptions {
///             wd: PathBuf::from("."),
///             variables: vars,
///             shell_commands: true,
///         })
///         .unwrap();
///     assert_eq!(rendered, "hello John. You're 132.3kg");
/// # Ok(())
/// }
pub type Template = Vec<TemplatePart>;

pub trait Render {
    fn render(&self, op: &RenderOptions) -> Result<String, Error>;
}

#[derive(Default, Debug, Clone)]
pub struct RenderOptions {
    pub wd: PathBuf,
    pub variables: HashMap<String, String>,
    pub shell_commands: bool,
}

impl Render for TemplatePart {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        match self {
            TemplatePart::Lit(l) => Ok(l.to_string()),
            TemplatePart::Var(v) => op
                .variables
                .get(v)
                .map(|s| s.to_string())
                .context("No such variable in the RenderOptions"),
            TemplatePart::Time(t) => Ok(Local::now().format(t).to_string()),
            TemplatePart::Cmd(c) => {
                let cmd = c.render(op)?;
                if op.shell_commands {
                    cmd_output(&cmd, &op.wd)
                } else {
                    Ok(format!("$({cmd})"))
                }
            }
            TemplatePart::Any(a) => a
                .iter()
                .filter_map(|p| p.render(op).ok())
                .next()
                .context("None of the alternatives given were found"),
        }
    }
}

impl Render for Template {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        self.iter()
            .map(|p| p.render(op))
            .collect::<Result<Vec<String>, Error>>()
            .map(|v| v.join(""))
    }
}

fn parse_single_part(part: &str) -> TemplatePart {
    if LITERAL_REPLACEMENTS.contains(&part) {
        TemplatePart::Lit(part.to_string())
    } else if part.starts_with(LITERAL_VALUE_QUOTE_CHAR) && part.ends_with(LITERAL_VALUE_QUOTE_CHAR)
    {
        TemplatePart::Lit(part[1..(part.len() - 1)].to_string())
    } else if part.starts_with(TIME_FORMAT_CHAR) {
        TemplatePart::Time(part.to_string())
    } else {
        TemplatePart::Var(part.to_string())
    }
}

fn parse_variables(templ: &str) -> Template {
    let mut last_match = 0usize;
    let mut template_parts = Vec::new();
    for cap in VARIABLE_REGEX.captures_iter(templ) {
        let m = cap.get(0).unwrap();
        template_parts.push(TemplatePart::Lit(templ[last_match..m.start()].to_string()));

        let cg = cap.get(1).unwrap();
        let cap_slice = &templ[cg.start()..cg.end()];
        if cap_slice.contains(OPTIONAL_RENDER_CHAR) {
            let parts = cap_slice
                .split(OPTIONAL_RENDER_CHAR)
                .map(|s| s.trim())
                .map(parse_single_part)
                .collect();

            template_parts.push(TemplatePart::Any(parts));
        } else {
            template_parts.push(parse_single_part(cap_slice));
        }
        last_match = m.end();
    }
    if !templ[last_match..].is_empty() {
        template_parts.push(TemplatePart::Lit(templ[last_match..].to_string()));
    }

    template_parts
}

/// Parses the template from string and makes a [`Template`]. Which you can render later./// Main Template that get's passed around, consists of `[Vec`] of [`TemplatePart`]
///
/// ```rust
/// # use std::error::Error;
/// # use std::collections::HashMap;
/// # use std::path::PathBuf;
/// # use string_template_plus::{Render, RenderOptions, parse_template};
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let templ = parse_template("hello {nickname?name?}. You're $(printf \"%.1f\" {weight})kg").unwrap();
///     let parts = concat!("[Lit(\"hello \"), ",
///                         "Any([Var(\"nickname\"), Var(\"name\"), Lit(\"\")]), ",
///                         "Lit(\". You're \"), ",
///                         "Cmd([Lit(\"printf \\\"%.1f\\\" \"), Var(\"weight\")]), ",
///                         "Lit(\"kg\")]");
///     assert_eq!(parts, format!("{:?}", templ));
/// # Ok(())
/// }
pub fn parse_template(templ_str: &str) -> Result<Template, Error> {
    let mut last_match = 0usize;
    let mut template_parts = Vec::new();
    for cmd in SHELL_COMMAND_REGEX.captures_iter(templ_str) {
        let m = cmd.get(0).unwrap();
        let cmd = cmd.get(1).unwrap();
        let mut pts = parse_variables(&templ_str[last_match..m.start()]);
        template_parts.append(&mut pts);
        template_parts.push(TemplatePart::Cmd(parse_variables(
            &templ_str[cmd.start()..cmd.end()],
        )));
        last_match = m.end();
    }
    let mut pts = parse_variables(&templ_str[last_match..]);
    template_parts.append(&mut pts);
    Ok(template_parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lit() {
        let templ = parse_template("hello name").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello name");
    }

    #[test]
    fn test_vars() {
        let templ = parse_template("hello {name}").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello world");
    }

    #[test]
    #[should_panic]
    fn test_novars() {
        let templ = parse_template("hello {name}").unwrap();
        let vars: HashMap<String, String> = HashMap::new();
        templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
    }

    #[test]
    fn test_novars_opt() {
        let templ = parse_template("hello {name?}").unwrap();
        let vars: HashMap<String, String> = HashMap::new();
        let rendered = templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello ");
    }

    #[test]
    fn test_optional() {
        let templ = parse_template("hello {age?name}").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello world");
    }

    #[test]
    fn test_special_chars() {
        let templ = parse_template("$hello {}? {{}{}}%").unwrap();
        let rendered = templ.render(&RenderOptions::default()).unwrap();
        assert_eq!(rendered, "$hello ? {}%");
    }

    #[test]
    fn test_optional_lit() {
        let templ = parse_template("hello {age?\"20\"}").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                variables: vars,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello 20");
    }

    #[test]
    fn test_command() {
        let templ = parse_template("hello $(echo {name})").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                wd: PathBuf::from("."),
                variables: vars,
                shell_commands: true,
            })
            .unwrap();
        assert_eq!(rendered, "hello world\n");
    }

    #[test]
    fn test_time() {
        let templ = parse_template("hello {name} at {%Y-%m-%d}").unwrap();
        let timefmt = Local::now().format("%Y-%m-%d");
        let output = format!("hello world at {}", timefmt);
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                wd: PathBuf::from("."),
                variables: vars,
                shell_commands: false,
            })
            .unwrap();
        assert_eq!(rendered, output);
    }

    #[test]
    fn test_var_or_time() {
        let templ = parse_template("hello {name} at {age?%Y-%m-%d}").unwrap();
        let timefmt = Local::now().format("%Y-%m-%d");
        let output = format!("hello world at {}", timefmt);
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = templ
            .render(&RenderOptions {
                wd: PathBuf::from("."),
                variables: vars,
                shell_commands: false,
            })
            .unwrap();
        assert_eq!(rendered, output);
    }
}
