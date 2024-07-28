#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use tomlp::drafts::parsetools::ParserLine;
use tomlp::drafts::tomlparse::TOMLParser;

const FILE: &str = "test_resources/blank.toml";
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init(FILE)?;
    let eof = "\n".to_string();
    let fire = "u0001f525".to_string();
    let next_nonws = "    \n\n\nt".to_string();

    assert_eq!(
        '🔥',
        parser
            .parse_multi_escape_sequence(ParserLine::new(fire, 0))?
            .0
    );
    assert_eq!(
        't',
        parser
            .parse_multi_escape_sequence(ParserLine::new(next_nonws, 0))?
            .0
    );
    assert_eq!(
        true,
        parser
            .parse_multi_escape_sequence(ParserLine::new(eof, 0))
            .is_err()
    );
    Ok(())
}
