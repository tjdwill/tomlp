use tomlp::drafts::tomlparse::{TOMLParser, TOMLTable};

fn main() -> Result<(), String> {
    let mut table = TOMLTable::new();
    let table_head = &mut table;
    let mut parser = TOMLParser::init("test_resources/keyvals.toml")?;
    while let Ok(pline) = parser.next_parserline() {
        let (key_val, _) = parser.parse_keyval(pline)?;
        TOMLParser::insert(key_val, table_head)?;
    }
    println!("Parsed Table: {:?}", table);
    Ok(())
}
