use std::env;
use std::error::Error;
use string_template_plus::{Render, Template};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        eprintln!("Provide template files to parse");
        return Ok(());
    }
    for filepath in args[1..].iter() {
        println!("*** {} ***", filepath);
        let contents = std::fs::read_to_string(filepath)?;
        let templ = Template::parse_template(&contents)?;
        templ.print();
        println!();
        println!("--------");
        println!("{:?}", templ.parts());
    }
    Ok(())
}
