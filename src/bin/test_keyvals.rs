use tomlp::drafts::tomlparse::{TOMLParser, KeyVal};

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test_resources/keyvals.toml")?;
    let mut successes = 0;
    while let Ok(pline) = parser.next_parserline() {
        let (KeyVal(key, val), _) = parser.parse_keyval(pline)?;
        println!("Key: {key:?}\nVal: {val:?}\n");
        successes += 1;
    }
    assert!(successes != 0);
    Ok(())
}
