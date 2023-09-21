use anyhow::{Context, Error};
use chrono::Local;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use subprocess::Exec;

pub static OPTIONAL_RENDER_CHAR: char = '?';
pub static TIME_FORMAT_CHAR: char = '%';
pub static LITERAL_VALUE_QUOTE_CHAR: char = '"';
pub static LITERAL_REPLACEMENTS: [&str; 6] = [
    "",  // to replace {} as empty string.
    "{", // to replace {{} as {
    "}", // to replace {}} as }
    "?", "%", "\"", // to use these chars without their special meanings
];

lazy_static! {
    pub static ref VARIABLE_REGEX: Regex = Regex::new(r"\{(.*?)\}").unwrap();
    pub static ref SHELL_COMMAND_REGEX: Regex = Regex::new(r"[$]\((.*?)\)").unwrap();
}

fn cmd_output(cmd: &str, wd: &PathBuf) -> Result<String, Error> {
    let mut out: String = String::new();
    Exec::shell(cmd)
        .cwd(wd)
        .stream_stdout()?
        .read_to_string(&mut out)?;
    Ok(out)
}

#[derive(Debug)]
pub enum TemplatePart<'a> {
    Lit(&'a str),
    Var(&'a str),
    Time(&'a str),
    Cmd(Vec<TemplatePart<'a>>),
    Any(Vec<TemplatePart<'a>>),
}

pub type Template<'a> = Vec<TemplatePart<'a>>;

pub trait Render {
    fn render(&self, op: &RenderOptions) -> Result<String, Error>;
}

#[derive(Default)]
pub struct RenderOptions {
    pub wd: PathBuf,
    pub variables: HashMap<String, String>,
}

impl<'a> Render for TemplatePart<'a> {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        match self {
            TemplatePart::Lit(l) => Ok(l.to_string()),
            TemplatePart::Var(v) => op
                .variables
                .get(*v)
                .map(|s| s.to_string())
                .context("No such variable in the RenderOptions"),
            TemplatePart::Time(t) => Ok(Local::now().format(t).to_string()),
            TemplatePart::Cmd(c) => cmd_output(&c.render(op)?, &op.wd),
            TemplatePart::Any(a) => a
                .iter()
                .filter_map(|p| p.render(op).ok())
                .next()
                .context("None of the alternatives given were found"),
        }
    }
}

impl<'a> Render for Template<'a> {
    fn render(&self, op: &RenderOptions) -> Result<String, Error> {
        self.iter()
            .map(|p| p.render(op))
            .collect::<Result<Vec<String>, Error>>()
            .map(|v| v.join(""))
    }
}

fn parse_single_part(part: &str) -> TemplatePart {
    if LITERAL_REPLACEMENTS.contains(&part) {
        // the input_map.get() is not working for "", idk why
        TemplatePart::Lit("")
    } else if part.starts_with(LITERAL_VALUE_QUOTE_CHAR) && part.ends_with(LITERAL_VALUE_QUOTE_CHAR)
    {
        TemplatePart::Lit(&part[1..(part.len() - 1)])
    } else if part.starts_with(TIME_FORMAT_CHAR) {
        TemplatePart::Time(part)
    } else {
        TemplatePart::Var(part)
    }
}

fn parse_variables(templ: &str) -> Template {
    let mut last_match = 0usize;
    let mut template_parts = Vec::new();
    for cap in VARIABLE_REGEX.captures_iter(templ) {
        let m = cap.get(0).unwrap();
        template_parts.push(TemplatePart::Lit(&templ[last_match..m.start()]));

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
    template_parts.push(TemplatePart::Lit(&templ[last_match..]));

    template_parts
}

pub fn parse_template(templ_str: &str) -> Result<Template, String> {
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
            })
            .unwrap();
        assert_eq!(rendered, output);
    }
}
