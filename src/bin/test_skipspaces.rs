use tomlp::prototype::*;
fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init("test.toml")?;
    println!("{:?}", parser);
    while parser.fill_buffer()? {
        println!("Parser Object:\n\t{:?}", parser);

        println!("Current Line: {}", parser.context.view_line());
        parser.skip_leading_ws();
        if parser.process_comment()? {
            println!("Found Comment!\n");
        };
        println!(
            "Graphemes: {:?}", parser.context.skipped_iter().collect::<Vec<&str>>()
        );
        println!("Parser Object:\n\t{:?}\n", parser);
    }
    println!("\nSuccessful Exit.");
    Ok(())
}
