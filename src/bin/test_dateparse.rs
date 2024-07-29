use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/dates.toml")?;
    println!("\nDate Parsing:");
    while let Ok(pline) = parser.next_parserline() {
        let (date, _pline) = TOMLParser::parse_date(pline)?;
        println!("{:?}", date);
    }
    Ok(())
}
