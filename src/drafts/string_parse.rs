/// This document is a draft of parsing TOML strings, outputting a Rust
/// String in the process.

#![allow(unused_imports, unused_variables)]
fn main() -> Result<(), i32> {
    let cmd = "   prgm -r -yhw --color file_path";
    let basic_str = "  \t  \"This is a string.\"";
    match process_basic_str(basic_str) {
        Some(output) => {
            println!("Success! String:\n{}", output);
        }
        None => return Err(1)
    }

    let literal_str = "  \t  \'This is a literal string.\'";
    match process_literal_str(literal_str) {
        Some(output) => {
            println!("Success! String:\n{}", output);
        }
        None => return Err(2)
    }

    Ok(())
}

/// Processing a String
use unicode_segmentation::{UnicodeSegmentation as utf8, Graphemes};

fn process_basic_str(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut graphemes = utf8::graphemes(input, true).peekable();

    let mut skips = 0;
    while let Some(&" ") | Some(&"\t") = graphemes.peek(){
        graphemes.next();
        skips += 1;
    }
    println!("Whitespace skipped: {}", skips);

    if let Some(&"\"") = graphemes.peek() {
        graphemes.next();
        loop {
            if let Some(&"\"") = graphemes.peek() {
                break
            } else {
                // Other checks here
                out.push_str(graphemes.next().unwrap());
            }
        }
        Some(out)
    } else{
        None
    }
} 

fn process_literal_str(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut graphemes = utf8::graphemes(input, true).peekable();

    let mut skips = 0;
    while let Some(&" ") | Some(&"\t") = graphemes.peek(){
        graphemes.next();
        skips += 1;
    }
    println!("Whitespace skipped: {}", skips);

    if let Some(&"\'") = graphemes.peek() {
        graphemes.next();
        loop {
            if let Some(&"\'") = graphemes.peek() {
                break
            } else {
                // Other checks here
                out.push_str(graphemes.next().unwrap());
            }
        }
        Some(out)
    } else{
        None
    }
}
