use super::parsetooling::{ParserLine, TOMLSeg};

fn skip_ws(mut seg: TOMLSeg<'_>) -> TOMLSeg<'_> {
    loop {
        if is_whitespace(seg.peek()) {
            seg.next();
        } else {
            break;
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

///////////////
// Helper Funcs
///////////////

fn invalid_comment_char(s: &str) -> bool {
    // produced via `print_invalid_comment_chars`
    const CHARS: [&str; 32] = [
        "\0", "\u{1}", "\u{2}", "\u{3}", "\u{4}", "\u{5}", "\u{6}", "\u{7}", "\u{8}", "\n",
        "\u{b}", "\u{c}", "\r", "\u{e}", "\u{f}", "\u{10}", "\u{11}", "\u{12}", "\u{13}", "\u{14}",
        "\u{15}", "\u{16}", "\u{17}", "\u{18}", "\u{19}", "\u{1a}", "\u{1b}", "\u{1c}", "\u{1d}",
        "\u{1e}", "\u{1f}", "\u{7f}",
    ];

    for c in CHARS {
        if s == c {
            return true;
        }
    }
    false
}
fn get_graphemes(s: &str) -> Vec<&str> {
    unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect::<Vec<_>>()
}
/// Use this to print an array of invalid chars. Make the resulting array via Copy/Paste.
pub fn print_invalid_comment_chars() {
    println!("Invalid TOML Comment Char Report");
    let inval_string = get_invalid_comment_chars();
    let invalids = get_graphemes(inval_string.as_str());
    println!("Total Num of Invalids: {}", invalids.len());
    println!("Invalid Comment Chars: {:?}\n", invalids);
}

fn get_invalid_comment_chars() -> String {
    let range1 = 0_u8..=8_u8;
    let range2 = u8::from_str_radix("A", 16).unwrap()..=u8::from_str_radix("1F", 16).unwrap();
    let range3 = u8::from_str_radix("7F", 16).unwrap()..u8::from_str_radix("80", 16).unwrap();
    let chars = range1.chain(range2.chain(range3)).collect::<Vec<u8>>();
    String::from_utf8(chars).unwrap()
}
