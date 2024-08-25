use tomlp::drafts::{tokens::TOMLTable, tomlparse::TOMLParser};

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/table_heads.toml")?;

    let mut table = TOMLTable::new();
    while let Ok(pline) = parser.next_parserline() {
        parser.parse_table_header(pline, &mut table)?;
        // println!("Table:\n{:?}", parser.view_table());
    }
    println!("Table:\n{:?}", &table); 
    Ok(())
}
