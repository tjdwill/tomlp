use tomlp;

fn main() -> Result<(), String> {
    let result = tomlp::parse("../tomlparse/test_resources/ripgrep.toml")?;
    println!("Debugged:i\n{:?}\n", result);
    println!("Formatted:\n{}", result);
    Ok(())
}
