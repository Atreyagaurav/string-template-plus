use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use subprocess::Exec;

pub static OPTIONAL_RENDER_CHAR: char = '?';
pub static LITERAL_VALUE_QUOTE_CHAR: char = '"';
pub static LITERAL_REPLACEMENTS: [&str; 3] = [
    "",  // to replace {} as empty string.
    "{", // to replace {{} as {
    "}", // to replace {}} as }
];

lazy_static! {
    pub static ref VARIABLE_REGEX: Regex = Regex::new(&format!(r"\{{(.*?)\}}")).unwrap();
    pub static ref SHELL_COMMAND_REGEX: Regex = Regex::new(&format!(r"[$]\((.*?)\)")).unwrap();
}

fn cmd_output(cmd: &str, wd: &PathBuf) -> Result<String, String> {
    let mut out: String = String::new();
    Exec::shell(cmd)
        .cwd(wd)
        .stream_stdout()
        .map_err(|e| e.to_string())?
        .read_to_string(&mut out)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

#[derive(Debug)]
pub enum TemplatePart<'a> {
    Lit(&'a str),
    Var(&'a str),
    Cmd(Vec<TemplatePart<'a>>),
    Any(Vec<TemplatePart<'a>>),
}

pub type Template<'a> = Vec<TemplatePart<'a>>;

#[derive(Default)]
pub struct RenderOptions {
    pub wd: PathBuf,
    pub variables: HashMap<String, String>,
}

impl<'a> TemplatePart<'a> {
    fn render(&self, op: &RenderOptions) -> String {
        match self {
            TemplatePart::Lit(l) => l.to_string(),
            TemplatePart::Var(v) => op.variables.get(*v).unwrap().to_string(),
            TemplatePart::Cmd(c) => cmd_output(&render_template(c, op), &op.wd).unwrap(),
            TemplatePart::Any(a) => a.iter().map(|p| p.render(op)).next().unwrap(),
        }
    }
}

fn render_template(templ: &Template, op: &RenderOptions) -> String {
    templ
        .iter()
        .map(|p| p.render(op))
        .collect::<Vec<String>>()
        .join("")
}

fn parse_variables(templ: &str) -> Template {
    let mut last_match = 0usize;
    let mut template_parts = Vec::new();
    for cap in VARIABLE_REGEX.captures_iter(&templ) {
        let m = cap.get(0).unwrap();
        template_parts.push(TemplatePart::Lit(&templ[last_match..m.start()]));

        let cg = cap.get(1).unwrap();
        let cap_slice = &templ[cg.start()..cg.end()];
        if cap_slice.contains(OPTIONAL_RENDER_CHAR) {
            let mut parts = Vec::new();
            for csg in cap_slice.split(OPTIONAL_RENDER_CHAR).map(|s| s.trim()) {
                if LITERAL_REPLACEMENTS.contains(&csg) {
                    // the input_map.get() is not working for "", idk why
                    parts.push(TemplatePart::Lit(""));
                } else if csg.starts_with(LITERAL_VALUE_QUOTE_CHAR)
                    && csg.ends_with(LITERAL_VALUE_QUOTE_CHAR)
                {
                    parts.push(TemplatePart::Lit(&csg[1..(csg.len() - 1)]));
                } else {
                    parts.push(TemplatePart::Var(&csg));
                }
            }

            template_parts.push(TemplatePart::Any(parts));
        } else {
            template_parts.push(TemplatePart::Var(cap_slice));
        }
        last_match = m.end();
    }
    template_parts.push(TemplatePart::Lit(&templ[last_match..]));

    template_parts
}

pub fn parse_template(templ_str: &str) -> Result<Template, String> {
    let mut last_match = 0usize;
    let mut template_parts = Vec::new();
    for cmd in SHELL_COMMAND_REGEX.captures_iter(&templ_str) {
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
    fn test_novars() {
        let templ = parse_template("hello name").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = render_template(
            &templ,
            &RenderOptions {
                variables: vars,
                ..Default::default()
            },
        );
        assert_eq!(rendered, "hello name");
    }

    #[test]
    fn test_vars() {
        let templ = parse_template("hello {name}").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = render_template(
            &templ,
            &RenderOptions {
                variables: vars,
                ..Default::default()
            },
        );
        assert_eq!(rendered, "hello world");
    }

    #[test]
    fn test_command() {
        let templ = parse_template("hello $(echo {name})").unwrap();
        let mut vars: HashMap<String, String> = HashMap::new();
        vars.insert("name".into(), "world".into());
        let rendered = render_template(
            &templ,
            &RenderOptions {
                wd: PathBuf::from("."),
                variables: vars,
            },
        );
        assert_eq!(rendered, "hello world\n");
    }
}
