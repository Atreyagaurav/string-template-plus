# Introduction
This is a simple template tool that works with variable names and
`HashMap` of `String`. The `Template` can be parsed from `str`
and then you can render it using the variables in `HashMap` and any
shell commands running through `Exec`.
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
- Limited formatting support like UPCASE, downcase, float significant digits, etc.
# Bug
Using transformations with `()` inside a command `$()` is not possible as they are recognized using regex. Need to fix it later.
# Usages
Simple variables:
```rust
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
```
Safe replace, blank if not present, or literal string if not present:
```rust
let templ = parse_template("hello {name?} {lastname?\"User\"}").unwrap();
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
```
Custom Commands:
```rust
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
```
You can turn off Custom Commands for safety:
```rust
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
```
Date Time:
```rust
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
```
# Transformers:
Although there is no format strings, there are transformer functions that can format for a bit. I'm planning to add more format functions as the need arises.
```rust
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
 let templ = parse_template(t).unwrap();
 let rendered = templ.render(&options).unwrap();
 assert_eq!(rendered, r);
 }
```

There are a few transformers available:

| Transformer          | Arguments | Function                 | Example                  |
|----------------------|-----------|--------------------------|--------------------------|
| f [`format_float`]   | [.]N      | only N number of decimal | {"1.12":f(.1)} ⇒ 1.1     |
| case [`string_case`] | up        | UPCASE a string          | {"na":case(up)} ⇒ NA     |
| case [`string_case`] | down      | downcase a string        | {"nA":case(down)} ⇒ na   |
| case [`string_case`] | proper    | Upcase the first letter  | {"nA":case(proper)} ⇒ Na |
| case [`string_case`] | title     | Title Case the string    | {"na":case(title)} ⇒ Na  |
| calc                 | [+-*/^]N  | Airthmatic calculation   | {"1":calc(+1*2^2)} ⇒ 16  |
| calc                 | [+-*/^]N  | Airthmatic calculation   | {"1":calc(+1,-1)} ⇒ 2,0  |
| count                | str       | count str occurance      | {"nata":count(a)} ⇒ 2    |

You can chain transformers ones after another for combined actions. For example, `count( ):calc(+1)` will give you total number of words in a sentence. 

# Render Iter
Makes a `RenderIter<'a>` that can generate incremented strings from the given `Template` and the `RenderOptions`. The Iterator will have `-N` appended where N is the number representing the number of instance.

```rust
let templ = parse_template("hello {name}").unwrap();
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
```

# Limitations
- You cannot use positional arguments in this template system, only named ones. `{}` will be replaced with empty string. Although you can use `"0"`, `"1"`, etc as variable names in the template and the render options variables.
- I haven't tested variety of names, although they should work try to keep the names identifier friendly.
- Currently doesn't have format specifiers, for now you can use the command options with `printf` bash command to format things the way you want, or use the transformers which have limited formatting capabilities.
Like a template `this is $(printf "%05.2f" {weight}) kg.` should be rendered with the correct float formatting.
