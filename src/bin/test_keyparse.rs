use tomlp::drafts::tomlparse;
use tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/keys.toml").unwrap();
    while let Ok(pline) = parser.next_parserline() {
        let (path, _) = parser.parse_key(pline)?;
        println!("Parsed Key: {:?}", path);
    }
    Ok(())
}
