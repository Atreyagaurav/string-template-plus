# string-template-plus

## Introduction

This is a simple template tool that works with variable names and
[`HashMap`] of [`String`]. The [`Template`] can be parsed from [`str`]
and then you can render it using the variables in [`HashMap`] and any
shell commands running through [`Exec`].

## Features
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


## Usages
Simple variables:
```rust
#
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
```

Safe replace, blank if not present, or literal string if not present:
```rust
#
let templ = Template::parse_template("hello {name?} {lastname?\"User\"}").unwrap();
let vars: HashMap<String, String> = HashMap::new();
let rendered = templ
.render(&RenderOptions {
variables: vars,
..Default::default()
            })
            .unwrap();
assert_eq!(rendered, "hello  User");
```

Alternate, whichever variable it finds first will be replaced:
```rust
#
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
```

Calculations can be written in lisp like language, it supports simple
functions. Using lisp can also allow you to write more complex
logic. The lisp implementation is the one from
https://github.com/brundonsmith/rust_lisp refer to the README there
for the functionality.

To access the values in lisp you can use the following functions:
- `st+var` : the value as string,
- `st+num` the value as a number, and
- `st+has` true if value is present else false.

You need to quote the symbol to pass to the functions (e.g. (st+num 'total).

Else, you can just write the variables in braces like normal as well.

```rust
#
let templ = Template::parse_template("hello {nickname?name}. You've done =(/ (st+num 'task_done) (st+num 'task_total)) work.").unwrap();
let mut vars: HashMap<String, String> = HashMap::new();
vars.insert("name".into(), "world".into());
vars.insert("task_done".into(), "1".into());
vars.insert("task_total".into(), "4".into());
let rendered = templ
.render(&RenderOptions {
variables: vars,
..Default::default()
            })
            .unwrap();
        assert_eq!(rendered, "hello world. You've done 0.25 work.");
```

Custom Commands:
```rust
#
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
```

You can turn off Custom Commands for safety:
```rust
#
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
```

Date Time:
```rust
#
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
```

## Transformers:
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
#
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
```

## Limitations
- You cannot use positional arguments in this template system, only named ones. `{}` will be replaced with empty string. Although you can use `"0"`, `"1"`, etc as variable names in the template and the render options variables.
- I haven't tested variety of names, although they should work try to keep the names identifier friendly.
- Currently doesn't have format specifiers, for now you can use the command options with `printf` bash command to format things the way you want, or use the transformers which have limited formatting capabilities.
Like a template `this is $(printf "%05.2f" {weight}) kg.` should be rendered with the correct float formatting.

License: GPL-3.0-only
