#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use tomlp::drafts::parsetooling::{ParserLine, tomlparse::TOMLParser};

const FILE: &str = "test_resources/blank.toml";
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init(FILE)?;
    Ok(())
}