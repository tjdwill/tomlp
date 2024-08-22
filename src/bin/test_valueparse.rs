use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/values.toml")?;
    let mut successes = 0;
    while let Ok(pline) = parser.next_parserline() {
        parser.parse_value(pline)?;
        successes += 1;
    }
    assert!(successes != 0);
    Ok(())
}
