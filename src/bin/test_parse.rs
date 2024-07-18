use tomlp::prototype::*;
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/test.toml")?;
    let table = parser.parse()?;
    println!("{:?}", table);
    Ok(())
}