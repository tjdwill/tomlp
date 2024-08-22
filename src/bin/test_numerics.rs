use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/numerics.toml")?;

    while let Ok(pline) = parser.next_parserline() {
        TOMLParser::parse_numeric(pline)?;
    }
    Ok(())
}
