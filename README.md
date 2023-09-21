Template with support for optional variables and such.

# Introduction

This is a simple template tool that works with variable names and
`HashMap` of `String`. The `Template` can be parsed from
`str` and then you can render it using the variables in
`HashMap` and any shell commands running through `Exec`.

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

# Limitations
- You cannot use positional arguments in this template system, only named ones.
- I haven't tested variety of names, although they should work try to keep the names identifier friendly.
- Currently doesn't have format specifiers, for now you can use the command options with `printf` bash command to format things the way you want.
Like a template `this is $(printf "%.2f" {weight}) kg.` should be rendered with the correct float formatting.

# Example
```
let templ = parse_template("hello {nickname?name}. You're $(printf \"%.1f\" {weight})kg").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("name".into(), "John".into());
vars.insert("weight".into(), "132.3423".into());
let rendered = templ
    .render(&RenderOptions {
        wd: PathBuf::from("."),
        variables: vars,
    })
    .unwrap();
assert_eq!(rendered, "hello John. You're 132.3kg");
```
