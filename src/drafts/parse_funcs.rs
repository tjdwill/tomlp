#![allow(dead_code, unused_variables, unused_imports)]
use super::parsetooling::{ParserLine, TOMLSeg};

fn skip_ws(mut seg: TOMLSeg<'_>) -> TOMLSeg<'_> {
    loop {
        if is_whitespace(seg.peek()) {
            seg.next();
        } else {
            break
        }
    }
    seg
}



fn process_basic_escape_sequence(iter: &mut TOMLSeg<'_>) -> Option<char> {
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
                "u" | "U" => escape_utf8(iter),
                _ => None, // This is where you would check for non-whitespace in a multi-string context
            }
        }
        None => None,
    }
}

pub fn process_comment(mut pline: ParserLine) -> Result<(), String> {
    let line_num = pline.line_num;
    let mut seg = pline.next_seg().unwrap();
    loop {
        match seg.next() {
            Some(ch) => {
                if !is_valid_comment(ch) {
                    return Err(format!(
                        "Invalid Comment Character: {} on Line {}",
                        ch, line_num
                    ));
                }
            }
            None => {
                let _next_seg = {
                    match pline.peek() {
                        Some(iter) => iter,
                        None => return Ok(()),
                    }
                };
                return process_comment(pline);
            }
        }
    }
}

pub fn is_valid_comment(c: &str) -> bool {
    true
}

fn is_whitespace(query: Option<&&str>) -> bool {
    match query {
        Some(text) => {
            match *text {
                " " | "\t" => true,
                _ => false, // was the empty string
            }
        }
        None => false,
    }
}

/////////////////
// UTF8 Functions
/////////////////

fn escape_utf8(iter: &mut TOMLSeg<'_>) -> Option<char> {
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

/// determines if the next entry of a UTF8Peek Iterator is a hexadecimal value
fn is_hexdigit(query: Option<&&str>) -> bool {
    match query {
        Some(text) => {
            let char_opt = text.chars().next();
            match char_opt {
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