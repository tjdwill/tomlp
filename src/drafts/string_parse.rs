#![allow(unused_imports, unused_variables, dead_code)]
/// This document is a draft for parsing TOML strings. It outputs Rust Strings.

fn main() -> Result<(), i32> {
    let basic_str = "  \t  \"Basic \\t \\\\ \\u0001F525 \\U0001F525 string.\"";
    match process_basic_str(basic_str) {
        Some(output) => {
            println!("Basic String Test:\n{}\n", output);
        }
        None => return Err(1),
    }

    let literal_str = "  \t  \'Literal \\t \\n \\u0001F525 string.\'";
    match process_literal_str(literal_str) {
        Some(output) => {
            println!("Literal String Test:\n{}\n", output);
        }
        None => return Err(2),
    }

    Ok(())
}

/// Processing a String
use std::iter::Peekable;
use unicode_segmentation::{Graphemes, UnicodeSegmentation as utf8};

type UTF8Peek<'a> = Peekable<Graphemes<'a>>;

fn process_basic_str(input: &str) -> Option<String> {
    let mut out = String::new();
    let mut graphemes = utf8::graphemes(input, true).peekable();

    // Skip whitespace
    while is_whitespace(graphemes.peek()) {
        graphemes.next();
    }

    if let Some(&"\"") = graphemes.peek() {
        // basic string context
        graphemes.next();
        loop {
            match graphemes.next() {
                Some("\"") => break,
                Some("\\") => match process_basic_escape_sequence(&mut graphemes) {
                    Some(c) => out.push(c),
                    None => return None,
                },
                // TO-DO: Add checks for disallowed UTF-8 sequences.
                //
                Some(c) => out.push_str(c),
                None => return None,
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

    while is_whitespace(graphemes.peek()) {
        graphemes.next();
    }

    if let Some(&"\'") = graphemes.peek() {
        // literal string context
        graphemes.next();
        loop {
            match graphemes.next() {
                Some("\'") => break,
                Some(c) => out.push_str(c),
                None => return None,
            }
        }
        Some(out)
    } else {
        None
    }
}

fn process_multi_string(mut lines: Vec<String>) -> Option<String> {

    // This gets around str.lines() removing all of the newline characters.
    // newline characters are needed for multi-strings.
    // This adds back information that was removed, but only in the string context
    // since that's where it's required.
    for line in &mut lines {
        line.push_str("\n");
    }
    
    let mut output = String::new();
    let mut lines_iter = lines.iter();

    let curr_line = lines_iter.next();
    match &curr_line {
        None => return None,
        _ => ()
    }

    println!("\nInput:{:?}", &lines);
    // find opening delimiter
    let mut graphs = utf8_peek(curr_line.unwrap().as_str());
    let mut count = 0;
    while count < 3 {  // Looking for """ delimiter
        match graphs.next() {
            Some("\"") => count += 1,
            _ => return None
        }
    }
    println!("Found delimiter!");
    count = 0;

    // multi-string context
    if let Some(&"\n") = graphs.peek() {
        // trim immediate newline
        graphs.next();
    }

    let mut graphemes_added = 0;
    while count < 3 {
        graphs.peek();
        println!("Iterator dump: {:?}", &graphs);
        match graphs.next() {
            None => {
                // try to get the next line
                let next = lines_iter.next();
                match next {
                    Some(ln) => {
                        println!("<PMS> Line REFILL.");
                        graphs = utf8_peek(ln.as_str());
                        continue
                    }
                    None => return None  // end of file without closing delimiter
                }
            }
            Some(c) => {
                
                graphemes_added += 1;
                match c {
                    "\\" => {
                        match process_multi_escape_sequence(graphs, lines_iter) {
                            Some((ch, currln_iter, lns_iter)) => {
                                if ch == '\"' {
                                    count += 1;
                                } else {
                                    count = 0;
                                }
                                output.push(ch);
                                graphs = currln_iter;
                                lines_iter = lns_iter;
                            }
                            None => return None
                        }
                    }
                    "\"" => {
                        count += 1;
                        output.push_str(c);
                        println!("Current count: {}", count);
                    }
                    _ => {
                        count = 0;
                        output.push_str(c);
                    }
                }
                println!("{}\n", &output);
            }
        }
    }
    println!("\nPMS: Found closing delim.");
    // remove last three quotation marks
    let outstring = utf8_peek(output.as_str())
            .take(graphemes_added - 3)
            .collect::<String>();
    println!("{}", outstring);
    Some(outstring)
}

/// Produce a UTF8 escape literal from a given iterator
/// Assumes Unix OS so the newline can fit into a char.
/// In the future (when I get this thing working), I can adjust for platform support
/// which would likely require this signature to change to return &'static str OR pass
/// the String structure in directly to push the slice to it.
/// This function is for DRAFTING purposes ONLY. The signature will be different in the implementation.
fn process_multi_escape_sequence<'a, I>(
    mut currline_iter: UTF8Peek<'a>,
    lines_iter: I,
) -> Option<(char, UTF8Peek<'a>, I)>
where
    I: Iterator<Item = &'a String>,
{
    // Assume we have identified a backslash
    match currline_iter.next() {
        Some(c) => {
            let outchar: char;
            match c {
                "b" => outchar = '\u{0008}',
                "t" => outchar = '\t',
                "n" => outchar = '\n',
                "f" => outchar = '\u{000C}',
                "r" => outchar = '\r',
                "\"" => outchar = '\"',
                "\\" => outchar = '\\',
                "u" | "U" => match utf8_escape(&mut currline_iter) {
                    Some(c) => outchar = c,
                    None => return None
                },
                _ => {
                    if !c.chars().next().unwrap().is_whitespace() {
                        return None
                    } else {
                        // find next non-whitespace char
                        return get_nonwhitespace(currline_iter, lines_iter)
                    }
                }
            }
            Some((outchar, currline_iter, lines_iter))
        }
        None => get_nonwhitespace(currline_iter, lines_iter),
    }
}

fn get_nonwhitespace<'a, I>(mut currline_iter: UTF8Peek<'a>, mut lines_iter: I) -> Option<(char, UTF8Peek<'a>, I)>
where
    I: Iterator<Item = &'a String>,
{
    loop {
        match currline_iter.next() {
            Some(c) => {
                let test_char = c.chars().next().unwrap();
                if !test_char.is_whitespace() {
                    return Some((test_char, currline_iter, lines_iter));
                } else {
                    continue;
                }
            }
            None => {
                // End of line; try to get a new one
                match lines_iter.next() {
                    None => return None, // EoF
                    Some(ln) => {
                        println!("<getWS>: Next Line: {:?}", &ln);
                        return get_nonwhitespace(utf8_peek(ln.as_str()), lines_iter);
                    }
                }
            }
        }
    }
}

fn process_basic_escape_sequence(iter: &mut UTF8Peek) -> Option<char> {
    // Assume we have identified a backslash
    match iter.next() {
        Some(c) => {
            match c {
                "b" => Some('\u{0008}'),
                "t" => Some('\t'),
                "n" => Some('\n'),
                "f" => Some('\u{000C}'),
                "r" => Some('\r'),
                "\"" => Some('\"'),
                "\\" => Some('\\'),
                "u" | "U" => utf8_escape(iter),
                _ => None, // This is where you would check for non-whitespace in a multi-string context
            }
        }
        None => None,
    }
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
            hex_val = 16 * hex_val + u32::from_str_radix(digit, 16).unwrap();
        } else if digits_processed == MIN_SEQ_LENGTH {
            break;
        } else {
            return None;
        }
        digits_processed += 1;
    }

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
                Some(c) => match c {
                    '0'..='9' | 'A'..='F' | 'a'..='f' => true,
                    _ => false,
                },
                None => false, // was the empty string
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
                        _ => false, // was the empty string
                    }
                }
                None => false,
            }
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_multi_string() {
        let answer = "The quick brown fox jumps over the lazy dog.".to_string();
        let bad_string_input = "   \t\r  \n\t\r\n\n\n\nThe quick brown \\\t\nfox jumps over \\\nthe lazy dog.\\\n\t\n\"\"\"";
        let lines = bad_string_input
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        assert_eq!(None, process_multi_string(lines));
        
        let good_string_input = "\"\"\"\nThe quick brown \\\t\nfox jumps over \\\nthe lazy dog.\\\n\"\"\"";
        let lines = good_string_input
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        assert_eq!(Some(answer), process_multi_string(lines));

        let string_input = "\"\"\"\nRoses are red\nViolets are blue\"\"\"";
        let lines = string_input
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        assert_eq!(Some("Roses are red\nViolets are blue".to_string()), process_multi_string(lines));
    }
    #[test]
    fn test_process_multi_escape_sequence() {
        /*
         """\
        The quick brown \
        fox jumps over \
        the lazy dog.\
        """
        */
        let string = "   \t\r  \n\t\r\n\n\n\nThe quick brown \\\t\nfox jumps over \\\nthe lazy dog.\\\n\"\"\"";
        let lines = string
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let mut lines_iter = lines.iter();
        let currline_iter = utf8_peek(lines_iter.next().unwrap().as_str());
        assert_eq!(
            Some('T'),
            match process_multi_escape_sequence(currline_iter, lines_iter) {
                Some((c, currln_iter, lns_iter)) => Some(c),
                None => None
            }
        );
    }

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

