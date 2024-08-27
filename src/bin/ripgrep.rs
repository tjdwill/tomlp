fn main() -> Result<(), String> {
    use tomlp::{parse, ValFromTOMLKey};

    let result = parse("../tomlparse/test_resources/ripgrep.toml")?;
    println!("Parsed TOML Table:{}", result);

    // query the table.
    // Let's get an array
    println!("\nRetrieved Value:\n{:?}", result.retrieve("package\0keywords", "\0"));
    Ok(())
}
