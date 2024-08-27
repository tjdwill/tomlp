// tree.rs
// Test the tree printing of the TOML table
fn main() -> Result<(), String> {
    let result = tomlp::parse("../tomlparse/test_resources/test.toml")?;
    println!("Debugged:\n{:?}\n", result);
    println!("Formatted:{}", result);
    Ok(())
}
