#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use tomlp::drafts::tomlparse::TOMLParser;
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/test.toml")?;
    Ok(())
}
