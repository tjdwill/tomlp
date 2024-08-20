use tomlp::drafts::tomlparse::TOMLParser;

fn main () -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/table_heads.toml")?;
    while let Ok(pline) = parser.next_parserline() {
        parser.parse_table_header(pline)?;
        // println!("Table:\n{:?}", parser.view_table());
    }
    println!("Table:\n{:?}", parser.view_table());
    Ok(())
}