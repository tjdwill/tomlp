use tomlp::drafts::{
    constants::{STR_TOKEN, LITERAL_STR_TOKEN},
    tokens::TOMLType,
    tomlparse::TOMLParser, 
};

const FILE: &str = "test_resources/strings.toml";

fn main() -> Result<(), String> {
    let mut parser = TOMLParser::init(FILE)?;
    let mut pline = parser.next_parserline()?;
    
    // Basic Strings
    println!("\nBasic Strings");
    while let Some(&STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("\nLine {}: ", pline.line_num());
        let (outstring, context) = parser.process_string(pline)?;
        if let TOMLType::BasicStr(str) = outstring {
            let outstring = str;
            println!("Basic String\n{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()))
        }
        pline = parser.next_parserline()?;
    }
    
    // Multi-Strings
    println!("\nMulti-line Strings");
    pline = parser.next_parserline()?;
    while let Some(&STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.process_string(pline)?;
        if let TOMLType::MultiStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()))
        }
        pline = parser.next_parserline()?;
    }
    
    // Literal Strings
    println!("\nLiteral Strings");
    pline = parser.next_parserline()?;
    while let Some(&LITERAL_STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.process_literal_string(pline)?;
        if let TOMLType::LitStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()))
        }
        pline = parser.next_parserline()?;
    }

    // Multi-Literal Strings
    println!("\nMulti-line Literal Strings");
    pline = parser.next_parserline()?;
    while let Some(&LITERAL_STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.process_literal_string(pline)?;
        if let TOMLType::MultiLitStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()))
        }
        pline = parser.next_parserline()?;
    }
    
    Ok(())
}
