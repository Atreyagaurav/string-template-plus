use std::error::Error;
use string_template_plus::{Render, Template};

const TEST: [&str; 4] = [
    r#"hi there this {is?a?"test"} for $(a simple case {that?} {might} be "possible")"#,
    r#"hi looks {like?a?"test"} for $(this does {work:case(up)} now) (yay)"#,
    r#"more {formatting?} options on {%F} and \\latex\{command\}\{with {variable}\}, should work."#,
    r##"let's try {every:f(2)?and?"anything":case(title)} for $(a complex case {that?%F?} {might} be "possible")"##,
];


fn main() -> Result<(), Box<dyn Error>> {

    for (i, e) in TEST.iter().enumerate() {
        println!("*** test{i} ***");
        let templ = Template::parse_template(e)?;
        templ.print();
        println!();
        println!("--------");
        println!("{:?}", templ.parts());
        println!("\n");
    }

    Ok(())
}
