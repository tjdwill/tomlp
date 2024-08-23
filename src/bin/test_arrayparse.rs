use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/arrays.toml")?;
    while let Ok(pline) = parser.next_parserline() {
        let (array, _) = parser.parse_array(pline)?;
        println!("Array: {:?}", array);
    }
    Ok(())
}
