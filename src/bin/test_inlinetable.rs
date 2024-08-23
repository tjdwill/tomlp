use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/inline_tables.toml")?;
    while let Ok(pline) = parser.next_parserline() {
        let (table, _pline) = parser.parse_inline_table(pline)?;
        println!("Parsed Inline Table: {:?}", table);
    }
    Ok(())
}
