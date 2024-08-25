#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use tomlp::drafts::tomlparse::TOMLParser;

fn main() -> Result<(), String> {
    const FILES: [&str; 4] = [ "test_resources/test.toml", "test_resources/spec-example-1.toml", "test_resources/ripgrep.toml", "test_resources/blank.toml" ];
    //const FILES: [&str; 1] = [ "test_resources/blank.toml" ];


    for file in FILES {
        let mut parser = TOMLParser::init(file)?;
        let table = parser.parse_toml()?;
        println!("Parsed TOML:\n{:?}", table);
        println!("\n");
    }
    Ok(())
}
