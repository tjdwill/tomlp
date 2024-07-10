/// This document is a draft of parsing TOML strings, outputting a Rust
/// String in the process.

#![allow(unused_imports, unused_variables)]
fn main() -> Result<(), i32> {
    /*
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
    */
    let test_str = "01Z2FJA";

    let mut graphs = utf8_peek(test_str);
    while let Some(_) = graphs.peek() {
        let preview = graphs.peek();
        println!("{:?} is Hexadecimal: {}", preview, is_hexdigit(preview));
        graphs.next();
    }

    let vector: Vec<i32> = vec![];
    let iter = vector.iter();
    let mut test_iter = iter.clone().take(10);
    println!("{:?}", test_iter);

    let next = test_iter.next();
    println!("{:?}", next);
    println!("{:?}", iter);

    Ok(())
}

/// Processing a String
use std::iter::Peekable;
use unicode_segmentation::{Graphemes, UnicodeSegmentation as utf8};

type UTF8Peek<'a> = Peekable<Graphemes<'a>>;

fn process_basic_str(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut graphemes = utf8::graphemes(input, true).peekable();

    let mut skips = 0;
    while let Some(&" ") | Some(&"\t") = graphemes.peek() {
        graphemes.next();
        skips += 1;
    }
    println!("Whitespace skipped: {}", skips);

    if let Some(&"\"") = graphemes.peek() {
        graphemes.next();
        loop {
            if let Some(&"\"") = graphemes.peek() {
                break;
            } else {
                // Other checks here
                out.push_str(graphemes.next().unwrap());
            }
        }
        Some(out)
    } else {
        None
    }
}

fn process_literal_str(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut graphemes = utf8_peek(input);

    let mut skips = 0;
    while let Some(&" ") | Some(&"\t") = graphemes.peek() {
        graphemes.next();
        skips += 1;
    }
    println!("Whitespace skipped: {}", skips);

    if let Some(&"\'") = graphemes.peek() {
        graphemes.next();
        loop {
            if let Some(&"\'") = graphemes.peek() {
                break;
            } else {
                // Other checks here
                out.push_str(graphemes.next().unwrap());
            }
        }
        Some(out)
    } else {
        None
    }
}

/// Produce a UTF8 escape literal from a given iterator
fn process_escape_sequence(iter: &mut UTF8Peek) -> Option<char> {
    // Assume we have identified a backslash

    utf8_escape(iter)
}

fn utf8_escape(iter: &mut UTF8Peek) -> Option<char> {
    // try to find 4 or 8 hexadecimal digits
    const MIN_SEQ_LENGTH: i32 = 4;
    const MAX_SEQ_LENGTH: i32 = 8;

    let mut hex_val = 0_u32;
    let mut digits_processed = 0;

    while digits_processed < MAX_SEQ_LENGTH {
        if is_hexdigit(iter.peek()) {
            let digit = iter.next().unwrap();
            hex_val = 16*hex_val + u32::from_str_radix(digit, 16).unwrap();
        } else if digits_processed == MIN_SEQ_LENGTH {
            break
        } else {
            return None
        }
        digits_processed += 1;
    }

    println!("Calculated Value: {}", hex_val);
    std::char::from_u32(hex_val)
}

fn utf8_peek(line: &str) -> UTF8Peek {
    utf8::graphemes(line, true).peekable()
}

/// determines if the next entry of a UTF8Peek Iterator is a hexadecimal value
fn is_hexdigit(query: Option<&&str>) -> bool {
    match query {
        Some(text) => {
            let copt = text.chars().next();
            match copt {
                Some(c) => {
                    match c {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => true,
                        _ => false,
                    }
                }
                None => false
            }
        }
        None => false,
    }
}

fn is_whitespace(query: Option<&&str>) -> bool {
    match query {
        Some(text) => {
            let copt = text.chars().next();
            match copt {
                Some(c) => {
                    match c {
                        ' ' | '\t' => true,
                        _ => false,
                    }
                }
                None => false
            }
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_escape() {
        let slice = "0AE8";
        let mut iter = utf8_peek(slice);
        match utf8_escape(&mut iter) {
            Some(seq) => assert_eq!('\u{0AE8}', seq),
            None => panic!("Incorrect processing."),
        }

        let bad_slice_0 = "4A3";
        let mut iter = utf8_peek(bad_slice_0);
        if let None = utf8_escape(&mut iter) {
        } else {
            panic!("Sequence Length Error");
        }

        let bad_slice_1 = "4A378";
        let mut iter = utf8_peek(bad_slice_1);
        if let None = utf8_escape(&mut iter) {
        } else {
            panic!("Sequence Length Error");
        }

        let good_slice = "0001F525";
        let mut iter = utf8_peek(good_slice);
        match utf8_escape(&mut iter) {
            Some(seq) => assert_eq!('\u{1F525}', seq),
            None => panic!("Incorrect processing: longer sequence."),
        }
    }

    #[test]
    fn test_is_whitespace() {
        assert_eq!(false, is_whitespace(None));       
        assert_eq!(false, is_whitespace(Some(&"")));       
        assert_eq!(false, is_whitespace(Some(&"t")));       

        assert_eq!(true, is_whitespace(Some(&" ")));       
        assert_eq!(true, is_whitespace(Some(&"\t")));       
    }

    #[test]
    fn test_is_hexdigit() {
        assert_eq!(false, is_hexdigit(Some(&"")));
        assert_eq!(false, is_hexdigit(Some(&"T")));
        assert_eq!(false, is_hexdigit(Some(&"J")));
        assert_eq!(false, is_hexdigit(None));
        
        for i in 0..10 {
            let s = i.to_string();
            assert_eq!(true, is_hexdigit(Some(&s.as_str())));
        }
        
        for i in 'a'..'g' {
            let s = i.to_string();
            assert_eq!(true, is_hexdigit(Some(&s.as_str())));
        }
        
        for i in 'A'..'G' {
            let s = i.to_string();
            assert_eq!(true, is_hexdigit(Some(&s.as_str())));
        }
    }
}
