use tomlp::prototype::*;
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test.toml")?;
    println!("{:?}", parser);
    while parser.fill_buffer()? {
        println!("Parser Object:\n{:?}", parser);

        println!("Current Line: {}", parser.context.view_line());
        parser.skip_leading_ws();
        println!("Graphemes: {:?}\n", parser.context.skipped_iter().collect::<Vec<&str>>());
        parser.process_comment()?;
    }
    println!("\nSuccessful Exit.");
    Ok(())
}
