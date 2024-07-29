#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use tomlp::drafts::{
    tomlparse::TOMLParser,
    parsetools::ParserLine,
    tokens::TOMLType,
};

fn main() -> Result<(), String> {

    let mut parser = TOMLParser::init("test_resources/bool.toml")?;
    let mut test = false;
    while let Ok(pline) = parser.next_parserline() {
        let (boolean, pline) = TOMLParser::parse_bool(pline)?;
        if let TOMLType::Bool(val) = boolean {
            assert_eq!(val, !test);
            test = val;
        }
    }
    Ok(())

}

