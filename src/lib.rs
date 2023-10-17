/*!
# Introduction

This is a simple template tool that works with variable names and
[`HashMap`] of [`String`]. The [`Template`] can be parsed from [`str`]
and then you can render it using the variables in [`HashMap`] and any
shell commands running through [`Exec`].

# Features
- Parse the template from a `str` that's easy to write,
- Support for alternatives in case some variables are not present,
  Use `?` to separate the alternatives, uses whichever it can find first. If `?` is at the end, leaves it blank instead of erroring out.
- Support for literal strings inside the alternative options,
  You can use a literal string `"string"` enclosed in `"` as an alternative if you want to put something instead of blank at the end.
- Support for the date time format using `chrono`,
  You can use any format starting with `%` inside the variable placeholder `{}` to use a date time format supported by chrono.
- Support for any arbitrary commands, etc.
You can keep any command inside `$(` and `)` to run it and use the result in the template. You can use other format elements inside it.
- Support for iterating (incremented with -N) strings with the same template conditions,
- Limited formatting support like UPCASE, downcase, float significant digits, etc. Look into [`transformers`] for more info.


# Usages
Simple variables:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("hello {name}").unwrap();
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
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("hello {name?} {lastname?\"User\"}").unwrap();
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
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("hello {nickname?name}").unwrap();
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
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("L=$(printf \"%.2f\" {length})").unwrap();
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
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("L=$(printf \"%.2f\" {length})").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("length".into(), "12.342323".into());
let rendered = templ
.render(&RenderOptions {
wd: PathBuf::from("."),
variables: vars,
shell_commands: false,
            })
            .unwrap();
        assert_eq!(rendered, "L=$(printf %.2f 12.342323)");
# Ok(())
# }
```

Date Time:
```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use chrono::Local;
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let templ = Template::parse_template("hello {name} at {%Y-%m-%d}").unwrap();
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

# Transformers:
Although there is no format strings, there are transformer functions that can format for a bit. I'm planning to add more format functions as the need arises.

To apply a tranformer to a variable provide it after [`VAR_TRANSFORM_SEP_CHAR`] (currently ":") to a variable template.

There are a few transformers available:

| Transformer | Funtion                        | Arguments | Function                  | Example                  |
|-------------|--------------------------------|-----------|---------------------------|--------------------------|
| f           | [`transformers::float_format`] | [.]N      | only N number of decimal  | {"1.12":f(.1)} ⇒ 1.1     |
| case        | [`transformers::string_case`]  | up        | UPCASE a string           | {"na":case(up)} ⇒ NA     |
| case        | [`transformers::string_case`]  | down      | downcase a string         | {"nA":case(down)} ⇒ na   |
| case        | [`transformers::string_case`]  | proper    | Upcase the first letter   | {"nA":case(proper)} ⇒ Na |
| case        | [`transformers::string_case`]  | title     | Title Case the string     | {"na":case(title)} ⇒ Na  |
| calc        | [`transformers::calc`]         | [+-*\/^]N | Airthmatic calculation    | {"1":calc(+1*2^2)} ⇒ 16  |
| calc        | [`transformers::calc`]         | [+-*\/^]N | Airthmatic calculation    | {"1":calc(+1,-1)} ⇒ 2,0  |
| count       | [`transformers::count`]        | str       | count str occurance       | {"nata":count(a)} ⇒ 2    |
| repl        | [`transformers::replace`]      | str1,str2 | replace str1 by str2      | {"nata":rep(a,o)} ⇒ noto |
| q           | [`transformers::quote`]        | [str1]    | quote with str1, or ""    | {"nata":q()} ⇒ "noto"    |
| take        | [`transformers::take`]         | str,N     | take Nth group sep by str | {"nata":take(a,2)} ⇒ "t" |
| trim        | [`transformers::trim`]         | str       | trim the string with str  | {"nata":trim(a)} ⇒ "nat" |

You can chain transformers ones after another for combined actions. For example, `count( ):calc(+1)` will give you total number of words in a sentence.

Examples are in individual functions in [`transformers`].

```rust
# use std::error::Error;
# use std::collections::HashMap;
# use std::path::PathBuf;
# use chrono::Local;
# use string_template_plus::{Render, RenderOptions, Template};
#
# fn main() -> Result<(), Box<dyn Error>> {
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("length".into(), "120.1234".into());
vars.insert("name".into(), "joHN".into());
vars.insert("job".into(), "assistant manager of company".into());
let options = RenderOptions {
variables: vars,
..Default::default()
        };
let cases = [
("L={length}", "L=120.1234"),
("L={length:calc(+100)}", "L=220.1234"),
("L={length:f(.2)} ({length:f(3)})", "L=120.12 (120.123)"),
("hi {name:case(up)}", "hi JOHN"),
(
 "hi {name:case(proper)}, {job:case(title)}",
 "hi John, Assistant Manager of Company",
),
 ("hi {name:case(down)}", "hi john"),
];

for (t, r) in cases {
 let templ = Template::parse_template(t).unwrap();
 let rendered = templ.render(&options).unwrap();
 assert_eq!(rendered, r);
 }
# Ok(())
# }
```

# Limitations
- You cannot use positional arguments in this template system, only named ones. `{}` will be replaced with empty string. Although you can use `"0"`, `"1"`, etc as variable names in the template and the render options variables.
- I haven't tested variety of names, although they should work try to keep the names identifier friendly.
- Currently doesn't have format specifiers, for now you can use the command options with `printf` bash command to format things the way you want, or use the transformers which have limited formatting capabilities.
Like a template `this is $(printf "%05.2f" {weight}) kg.` should be rendered with the correct float formatting.
*/
use anyhow::Error;
use chrono::Local;
use colored::Colorize;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use subprocess::Exec;

pub mod errors;
pub mod transformers;

/// Character to separate the variables. If the first variable is not present it'll use the one behind it and so on. Keep it at the end, if you want a empty string instead of error on missing variable.
pub static OPTIONAL_RENDER_CHAR: char = '?';
/// Character that should be in the beginning of the variable to determine it as datetime format.
pub static TIME_FORMAT_CHAR: char = '%';
/// Character that separates variable with format
pub static VAR_TRANSFORM_SEP_CHAR: char = ':';
/// Quote characters to use to make a value literal instead of a variable. In combination with [`OPTIONAL_RENDER_CHAR`] it can be used as a default value when variable(s) is/are not present.
pub static LITERAL_VALUE_QUOTE_CHAR: char = '"';
/// Character to escape special meaning characters
pub static ESCAPE_CHAR: char = '\\';
/// Characters that should be replaced as themselves if presented as a variable
static LITERAL_REPLACEMENTS: [&str; 3] = [
    "",  // to replace {} as empty string.
    "{", // to replace {{} as {
    "}", // to replace {}} as }
];

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
/// [`TemplatePart::Var`] = Variable part like `"name"` in `"hi {name}"` and format specifier
/// [`TemplatePart::Time`] = Date time format like `"%F"` in `"Today: {%F}"`
/// [`TemplatePart::Cmd`] = Command like `"echo world"` in `"hello $(echo world)"`
/// [`TemplatePart::Any`] = Optional format like `"name?age"` in `"hello {name?age}"`
///
/// [`TemplatePart::Cmd`] and [`TemplatePart::Any`] can in turn contain other [`TemplatePart`] inside them. Haven't tested on nesting complex ones within each other though.
#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// Literal string, keep them as they are
    Lit(String),
    /// Variable and format, uses the variable's value in the rendered String
    Var(String, String),
    /// DateTime format, use [`chrono::Local`] in the given format
    Time(String),
    /// Shell Command, use the output of command in the rendered String
    Cmd(Vec<TemplatePart>),
    /// Multiple variables or [`TemplatePart`]s, use the first one that succeeds
    Any(Vec<TemplatePart>),
}

lazy_static! {
    pub static ref TEMPLATE_PAIRS_START: [char; 3] = ['{', '"', '('];
    pub static ref TEMPLATE_PAIRS_END: [char; 3] = ['}', '"', ')'];
    pub static ref TEMPLATE_PAIRS: HashMap<char, char> = TEMPLATE_PAIRS_START
        .iter()
        .zip(TEMPLATE_PAIRS_END.iter())
        .map(|(k, v)| (*k, *v))
        .collect();
}

impl TemplatePart {
    pub fn lit(part: &str) -> Self {
        Self::Lit(part.to_string())
    }
    pub fn var(part: &str) -> Self {
        if let Some((part, fstr)) = part.split_once(VAR_TRANSFORM_SEP_CHAR) {
            Self::Var(part.to_string(), fstr.to_string())
        } else {
            Self::Var(part.to_string(), "".to_string())
        }
    }

    pub fn time(part: &str) -> Self {
        Self::Time(part.to_string())
    }

    /// Parse a [`&str`] into [`TemplatePart::Lit`], [`TemplatePart::Time`], or [`TemplatePart::Var`]
    pub fn maybe_var(part: &str) -> Self {
        if LITERAL_REPLACEMENTS.contains(&part) {
            Self::lit(part)
        } else if part.starts_with(LITERAL_VALUE_QUOTE_CHAR)
            && part.ends_with(LITERAL_VALUE_QUOTE_CHAR)
        {
            Self::lit(&part[1..(part.len() - 1)])
        } else if part.starts_with(TIME_FORMAT_CHAR) {
            Self::time(part)
        } else {
            Self::var(part)
        }
    }

    pub fn cmd(parts: Vec<TemplatePart>) -> Self {
        Self::Cmd(parts)
    }

    pub fn parse_cmd(part: &str) -> Result<Self, errors::RenderTemplateError> {
        Self::tokenize(part).map(Self::cmd)
    }

    pub fn any(parts: Vec<TemplatePart>) -> Self {
        Self::Any(parts)
    }

    pub fn maybe_any(part: &str) -> Self {
        if part.contains(OPTIONAL_RENDER_CHAR) {
            let parts = part
                .split(OPTIONAL_RENDER_CHAR)
                .map(|s| s.trim())
                .map(Self::maybe_var)
                .collect();

            Self::any(parts)
        } else {
            Self::maybe_var(part)
        }
    }

    fn find_end(
        end: char,
        templ: &str,
        offset: usize,
    ) -> Result<usize, errors::RenderTemplateError> {
        if end == '"' {
            return templ[offset..].find(end).map(|i| i + offset).ok_or(
                errors::RenderTemplateError::InvalidFormat(
                    templ.to_string(),
                    "Quote not closed".to_string(),
                ),
            );
        }
        let mut nest: Vec<char> = Vec::new();
        for (i, c) in templ[offset..].chars().enumerate() {
            if c == end && nest.is_empty() {
                return Ok(offset + i);
            } else if TEMPLATE_PAIRS_START.contains(&c) {
                if c == '"' {
                    if nest.contains(&c) {
                        while Some('"') != nest.pop() {}
                        continue;
                    }
                }
                nest.push(c);
            } else if TEMPLATE_PAIRS_END.contains(&c) {
                if let Some(last) = nest.pop() {
                    if c != TEMPLATE_PAIRS[&last] {
                        return Err(errors::RenderTemplateError::InvalidFormat(
                            templ.to_string(),
                            format!("Extra {} at [{}] in template", c, offset + i),
                        ));
                    }
                } else {
                    return Err(errors::RenderTemplateError::InvalidFormat(
                        templ.to_string(),
                        format!("Extra {} at [{}] in template", c, offset + i),
                    ));
                }
            }
        }
        Err(errors::RenderTemplateError::InvalidFormat(
            templ.to_string(),
            format!(
                "Closing {} not found from [{}] onwards in template",
                end, offset,
            ),
        ))
    }
    pub fn tokenize(templ: &str) -> Result<Vec<Self>, errors::RenderTemplateError> {
        let mut parts: Vec<TemplatePart> = Vec::new();
        let mut last = 0usize;
        let mut i = 0usize;
        let mut escape = false;
        while i < templ.len() {
            if templ[i..].starts_with(ESCAPE_CHAR) {
                if !escape {
                    if i > last {
                        parts.push(Self::lit(&templ[last..i]));
                    }
                    i += 1;
                    last = i;
                    escape = true;
                    continue;
                }
            }
            if escape {
                parts.push(Self::lit(&templ[i..(i + 1)]));
                last = i + 1;
                i += 1;
                escape = false;
                continue;
            }
            if templ[i..].starts_with("$(") {
                let end = Self::find_end(')', templ, i + 2)?;
                if i > last {
                    parts.push(Self::lit(&templ[last..i]));
                }
                last = end + 1;
                parts.push(Self::parse_cmd(&templ[(i + 2)..end])?);
                i = end;
            } else if templ[i..].starts_with("{") {
                let end = Self::find_end('}', templ, i + 1)?;
                if i > last {
                    parts.push(Self::lit(&templ[last..i]));
                }
                last = end + 1;
                parts.push(Self::maybe_any(&templ[(i + 1)..end]));
                i = end;
            } else if templ[i..].starts_with("\"") {
                let end = Self::find_end('"', templ, i + 1)?;
                if i > last {
                    parts.push(Self::lit(&templ[last..i]));
                }
                last = end + 1;
                parts.push(Self::lit(&templ[(i + 1)..end]));
                i = end;
            }
            i += 1;
        }
        if templ.len() > last {
            parts.push(Self::lit(&templ[last..]));
        }
        Ok(parts)
    }

    pub fn variables(&self) -> Vec<&str> {
        match self {
            TemplatePart::Var(v, _) => vec![v.as_str()],
            TemplatePart::Any(any) => any.iter().map(|p| p.variables()).flatten().collect(),
            TemplatePart::Cmd(cmd) => cmd.iter().map(|p| p.variables()).flatten().collect(),
            _ => vec![],
        }
    }
}
impl ToString for TemplatePart {
    fn to_string(&self) -> String {
        match self {
            Self::Lit(s) => format!("{0}{1}{0}", LITERAL_VALUE_QUOTE_CHAR, s),
            Self::Var(s, _) => s.to_string(),
            Self::Time(s) => s.to_string(),
            Self::Cmd(v) => v
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(""),
            Self::Any(v) => v
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(OPTIONAL_RENDER_CHAR.to_string().as_str()),
        }
    }
}

/// Main Template that get's passed around, consists of `[Vec`] of [`TemplatePart`]
///
/// ```rust
/// # use std::error::Error;
/// # use std::collections::HashMap;
/// # use std::path::PathBuf;
/// # use string_template_plus::{Render, RenderOptions, Template};
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let templ = Template::parse_template("hello {nickname?name}. You're $(printf \"%.1f\" {weight})kg").unwrap();
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
#[derive(Debug, Clone)]
pub struct Template {
    original: String,
    parts: Vec<TemplatePart>,
}

impl Template {
    /// Parses the template from string and makes a [`Template`]. Which you can render later./// Main Template that get's passed around, consists of `[Vec`] of [`TemplatePart`]
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # use std::collections::HashMap;
    /// # use std::path::PathBuf;
    /// # use string_template_plus::{Render, RenderOptions, Template};
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    ///     let templ = Template::parse_template("hello {nickname?name?}. You're $(printf \\\"%.1f\\\" {weight})kg").unwrap();
    ///     let parts = concat!("[Lit(\"hello \"), ",
    ///                  "Any([Var(\"nickname\", \"\"), Var(\"name\", \"\"), Lit(\"\")]), ",
    ///                  "Lit(\". You're \"), ",
    ///                  "Cmd([Lit(\"printf \"), Lit(\"\\\"\"), Lit(\"%.1f\"), Lit(\"\\\"\"), Lit(\" \"), Var(\"weight\", \"\")]), ",
    ///                  "Lit(\"kg\")]");
    ///     assert_eq!(parts, format!("{:?}", templ.parts()));
    /// # Ok(())
    /// }
    pub fn parse_template(templ_str: &str) -> Result<Template, Error> {
        let template_parts = TemplatePart::tokenize(templ_str)?;
        Ok(Self {
            original: templ_str.to_string(),
            parts: template_parts,
        })
    }

    pub fn parts(&self) -> &Vec<TemplatePart> {
        &self.parts
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    /// Concatenated String if [`Template`] is only literal strings
    pub fn lit(&self) -> Option<String> {
        let mut lit = String::new();
        for part in &self.parts {
            if let TemplatePart::Lit(l) = part {
                lit.push_str(l);
            } else {
                return None;
            }
        }
        Some(lit)
    }
}

/// Provides the function to render the object with [`RenderOptions`] into [`String`]
pub trait Render {
    fn render(&self, op: &RenderOptions) -> Result<String, Error>;

    fn print(&self);
}

/// Options for the [`Template`] to render into [`String`]
#[derive(Default, Debug, Clone)]
pub struct RenderOptions {
    /// Working Directory for the Shell Commands
    pub wd: PathBuf,
    /// Variables to use for the template
    pub variables: HashMap<String, String>,
    /// Run Shell Commands for the output or not
    pub shell_commands: bool,
}

impl RenderOptions {
    pub fn render(&self, templ: &Template) -> Result<String, Error> {
        templ.render(self)
    }

    /// Makes a [`RenderIter<'a>`] that can generate incremented strings from the given [`Template`] and the [`RenderOptions`]. The Iterator will have `-N` appended where N is the number representing the number of instance.
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # use std::collections::HashMap;
    /// # use string_template_plus::{Render, RenderOptions, Template};
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    ///     let templ = Template::parse_template("hello {name}").unwrap();
    ///     let mut vars: HashMap<String, String> = HashMap::new();
    ///     vars.insert("name".into(), "world".into());
    ///     let options = RenderOptions {
    ///         variables: vars,
    ///         ..Default::default()
    ///     };
    ///     let mut names = options.render_iter(&templ);
    ///     assert_eq!("hello world-1", names.next().unwrap());
    ///     assert_eq!("hello world-2", names.next().unwrap());
    ///     assert_eq!("hello world-3", names.next().unwrap());
    /// # Ok(())
    /// # }
    pub fn render_iter<'a>(&'a self, templ: &'a Template) -> RenderIter<'a> {
        RenderIter {
            template: templ,
            options: self,
            count: 0,
        }
    }
}

/// Render option with [`Iterator`] support. You can use this to get
/// incremented render results. It'll add `-N` to the render
/// [`Template`] where `N` is the count (1,2,3...). It can be useful
/// to make files with a given template.
///
/// ```rust
/// # use std::error::Error;
/// # use std::collections::HashMap;
/// # use string_template_plus::{Render, RenderOptions, RenderIter, Template};
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
///     let templ = Template::parse_template("hello {name}").unwrap();
///     let mut vars: HashMap<String, String> = HashMap::new();
///     vars.insert("name".into(), "world".into());
///     let options = RenderOptions {
///         variables: vars,
///         ..Default::default()
///     };
///     let mut names = RenderIter::new(&templ, &options);
///     assert_eq!("hello world-1", names.next().unwrap());
///     assert_eq!("hello world-2", names.next().unwrap());
///     assert_eq!("hello world-3", names.next().unwrap());
/// # Ok(())
/// # }
#[derive(Debug, Clone)]
pub struct RenderIter<'a> {
    template: &'a Template,
    options: &'a RenderOptions,
    count: usize,
}

impl<'a> RenderIter<'a> {
    /// Creates a new [`RenderIter<'a>`] object
    pub fn new(template: &'a Template, options: &'a RenderOptions) -> Self {
        Self {
            template: &template,
            options: &options,
            count: 0,
        }
    }
}

impl<'a> Iterator for RenderIter<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.template.render(&self.options).ok().map(|t| {
            self.count += 1;
            format!("{}-{}", t, self.count)
        })
    }
}

impl Render for TemplatePart {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        match self {
            TemplatePart::Lit(l) => Ok(l.to_string()),
            TemplatePart::Var(v, f) => op
                .variables
                .get(v)
                .ok_or(errors::RenderTemplateError::VariableNotFound(v.to_string()))
                .map(|s| -> Result<String, Error> { Ok(transformers::apply_tranformers(s, f)?) })?,
            TemplatePart::Time(t) => Ok(Local::now().format(t).to_string()),
            TemplatePart::Cmd(c) => {
                let cmd = c.render(op)?;
                if op.shell_commands {
                    cmd_output(&cmd, &op.wd)
                } else {
                    Ok(format!("$({cmd})"))
                }
            }
            TemplatePart::Any(a) => a.iter().find_map(|p| p.render(op).ok()).ok_or(
                errors::RenderTemplateError::AllVariablesNotFound(
                    a.iter().map(|p| p.to_string()).collect(),
                )
                .into(),
            ),
        }
    }
    /// Visualize what has been parsed so it's easier to debug
    fn print(&self) {
        match self {
            Self::Lit(s) => print!("{}", s),
            Self::Var(s, sf) => print!("{}", {
                if sf.is_empty() {
                    s.on_blue()
                } else {
                    format!("{}:{}", s, sf.on_bright_blue()).on_blue()
                }
            }),
            Self::Time(s) => print!("{}", s.on_yellow()),
            Self::Cmd(v) => {
                // overline; so the literal values are detected
                print!("\x1B[53m");
                print!("{}", "$(".on_red());
                v.iter().for_each(|p| {
                    print!("\x1B[53m");
                    p.print();
                });
                print!("\x1B[53m");
                print!("{}", ")".on_red());
            }
            Self::Any(v) => {
                v[..(v.len() - 1)].iter().for_each(|p| {
                    // underline; so the literal values are detected
                    print!("\x1B[4m");
                    p.print();
                    print!("\x1B[4m");
                    print!("{}", OPTIONAL_RENDER_CHAR.to_string().on_yellow());
                });
                print!("\x1B[4m");
                v.iter().last().unwrap().print();
                print!("\x1B[0m");
            }
        }
    }
}

impl Render for Vec<TemplatePart> {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        self.iter()
            .map(|p| p.render(op))
            .collect::<Result<Vec<String>, Error>>()
            .map(|v| v.join(""))
    }

    fn print(&self) {
        self.iter().for_each(|p| p.print());
    }
}

impl Render for Template {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        self.parts.render(op)
    }

    fn print(&self) {
        self.parts.print();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lit() {
        let templ = Template::parse_template("hello name").unwrap();
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
        let templ = Template::parse_template("hello {name}").unwrap();
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
    fn test_vars_format() {
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("length".into(), "120.1234".into());
        vars.insert("name".into(), "joHN".into());
        vars.insert("job".into(), "assistant manager of company".into());
        let options = RenderOptions {
            variables: vars,
            ..Default::default()
        };
        let cases = [
            ("L={length}", "L=120.1234"),
            ("L={length:calc(+100)}", "L=220.1234"),
            ("L={length:count(.):calc(+1)}", "L=2"),
            ("L={length:f(.2)} ({length:f(3)})", "L=120.12 (120.123)"),
            ("hi {name:case(up)}", "hi JOHN"),
            (
                "hi {name:case(proper)}, {job:case(title)}",
                "hi John, Assistant Manager of Company",
            ),
            ("hi {name:case(down)}", "hi john"),
        ];

        for (t, r) in cases {
            let templ = Template::parse_template(t).unwrap();
            let rendered = templ.render(&options).unwrap();
            assert_eq!(rendered, r);
        }
    }

    #[test]
    #[should_panic]
    fn test_novars() {
        let templ = Template::parse_template("hello {name}").unwrap();
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
        let templ = Template::parse_template("hello {name?}").unwrap();
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
        let templ = Template::parse_template("hello {age?name}").unwrap();
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
        let templ = Template::parse_template("$hello {}? \\{\\}%").unwrap();
        let rendered = templ.render(&RenderOptions::default()).unwrap();
        assert_eq!(rendered, "$hello ? {}%");
    }

    #[test]
    fn test_special_chars2() {
        let templ = Template::parse_template("$hello {}? \"{\"\"}\"%").unwrap();
        let rendered = templ.render(&RenderOptions::default()).unwrap();
        assert_eq!(rendered, "$hello ? {}%");
    }

    #[test]
    fn test_optional_lit() {
        let templ = Template::parse_template("hello {age?\"20\"}").unwrap();
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
        let templ = Template::parse_template("hello $(echo {name})").unwrap();
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
    fn test_command_quote() {
        let templ = Template::parse_template("hello $(printf \\\"%s %d\\\" {name} {age})").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        vars.insert("age".into(), "1".into());
        let rendered = templ
            .render(&RenderOptions {
                wd: PathBuf::from("."),
                variables: vars,
                shell_commands: true,
            })
            .unwrap();
        assert_eq!(rendered, "hello world 1");
    }

    #[test]
    fn test_time() {
        let templ = Template::parse_template("hello {name} at {%Y-%m-%d}").unwrap();
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
        let templ = Template::parse_template("hello {name} at {age?%Y-%m-%d}").unwrap();
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
    fn test_render_iter() {
        let templ = Template::parse_template("hello {name}").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let options = RenderOptions {
            variables: vars,
            ..Default::default()
        };
        let mut names = options.render_iter(&templ);
        assert_eq!("hello world-1", names.next().unwrap());
        assert_eq!("hello world-2", names.next().unwrap());
        assert_eq!("hello world-3", names.next().unwrap());
    }
}
