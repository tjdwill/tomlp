use parsetooling::ParserLine;
use tomlp::drafts::{parsetooling, tomlparse};
use tomlparse::TOMLParser;

const FILE: &str = "test_resources/strings.toml";

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init(FILE)?;
    let mut pline = parser.next_parserline()?;
    let (outstring, context) = parser.process_string(pline)?;
    println!("{}", outstring);
    Ok(())
}
