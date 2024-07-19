// stdlib imports
#![allow(unused_mut)]
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::iter::Peekable;
use std::path::Path;
// third-party imports
use crate::drafts::constants::STR_TOKEN;
use unicode_segmentation::UnicodeSegmentation;

// my imports
use super::parsetooling::{ParserLine, TOMLSeg};

static EOF_ERROR: &str = "End of File during parsing operation.";

#[derive(Debug)]
pub struct TOMLParser {
    buffer: String,
    reader: BufReader<File>,
    line_num: usize,
}
impl TOMLParser {
    ////////////////////////
    // Creation/Modification
    ////////////////////////
    pub fn init(file_path: &str) -> Result<Self, String> {
        let fd = Self::validate_file(file_path)?;
        Ok(Self {
            buffer: String::with_capacity(100 * 4),
            reader: BufReader::new(fd),
            line_num: 0,
        })
    }

    fn validate_file(input: &str) -> Result<File, String> {
        use std::ffi::OsStr;

        let toml_ext: &OsStr = OsStr::new("toml");
        let test = Path::new(input);
        match test.extension() {
            Some(ext) => {
                if !test.exists() {
                    return Err("File does not exist.".to_string());
                } else if ext != toml_ext {
                    return Err("Incorrect file extension.".to_string());
                } else {
                }
            }
            None => return Err("Incorrect file extension.".to_string()),
        }

        match File::open(input) {
            Ok(fd) => Ok(fd),
            Err(err) => Err(format!("File Open Error: {}", err.kind())),
        }
    }

    /// returns false -> EoF
    /// Won't check for EoF mid-value parsing.
    /// I only plan to check in the outer loop.
    pub fn next_line(&mut self) -> Result<bool, String> {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => return Ok(false),
            Ok(sz) => {
                self.line_num += 1;
                return Ok(true);
            }
            Err(err) => {
                return Err(format!(
                    "Read error for line {1}: {0}",
                    err.kind(),
                    self.line_num + 1
                ))
            }
        }
    }

    fn curr_parserline(&self) -> ParserLine {
        ParserLine::new(self.buffer.clone(), self.line_num)
    }

    pub fn next_parserline(&mut self) -> Result<ParserLine, String> {
        if !self.next_line()? {
            return Err(String::from(EOF_ERROR));
        } else {
            Ok(self.curr_parserline())
        }
    }
    ////////////////////
    // Parsing Functions
    ////////////////////
    // Processing a string

    pub fn process_string(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(String, ParserLine), String> {
        // determine if the multi-string delimiter is present.

        // UNWRAP justification: a " character was found before calling this function, so we know the segment exists.
        let mut seg = pline.peek().unwrap();
        for i in 0..3 {
            if let Some(&"\"") = seg.peek() {
                seg.next();
                continue;
            }
            return self.process_basic_string(pline);
        }
        return self.process_multi_string(pline);
    }

    pub fn process_multi_string(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(String, ParserLine), String> {
        // throw away first three characters (the delimiter)
        let mut quote_count = 0;
        let sz = self.buffer.capacity();
        let mut grapheme_pool = String::with_capacity(sz);

        let mut seg = pline.next_seg().unwrap();
        for i in 0..3 {
            seg.next();
        }
        // trim immediate newline
        if let Some(&"\n") = seg.peek() {
            seg.next();
        }
        // in multi-string context
        let mut graphemes_added = 0;
        while quote_count < 3 {
            match seg.next() {
                Some(ch) => {
                    graphemes_added += 1;
                    match ch {
                        STR_TOKEN => {
                            quote_count += 1;
                            grapheme_pool.push_str(ch);
                        }

                        "\\" => {
                            // escape sequence
                            let count = seg.count();
                            let (ch, context, inc_delim) = self.process_multi_escape_sequence(
                                ParserLine::continuation(pline, count),
                            )?;

                            if inc_delim {
                                quote_count += 1;
                            } else {
                                quote_count = 0;
                            }

                            grapheme_pool.push(ch);

                            pline = context;
                            if pline.is_exhausted() {
                                pline = self.next_parserline()?;
                            }
                            seg = pline.next_seg().unwrap();
                        }

                        _ => {
                            quote_count = 0;
                            grapheme_pool.push_str(ch);
                        }
                    }
                }
                None => {
                    if pline.is_exhausted() {
                        pline = self.next_parserline()?;
                    }
                    seg = pline.next_seg().unwrap();
                }
            }
        } // possibly found closing delimiter

        // check for extra quotation mark (this is a really annoying thing to allow)
        // REFERENCE: https://toml.io/en/v1.0.0#string
        if let Some(&"\"") = seg.peek() {
            grapheme_pool.push_str(seg.next().unwrap());
            graphemes_added += 1;
        }

        let outstring = grapheme_pool
            .as_str()
            .graphemes(true)
            .take(graphemes_added - 3)
            .collect::<String>();
        let count = seg.count();
        let context = ParserLine::continuation(pline, count);
        Ok((outstring, context))
    }

    pub fn process_basic_string(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(String, ParserLine), String> {
        // Throw away first character (delimiter)
        let mut seg = pline.next_seg().unwrap();
        seg.next();
        let mut grapheme_pool = String::with_capacity(self.buffer.capacity());
        loop {
            match seg.next() {
                None => match pline.next_seg() {
                    None => {
                        return Err(format!(
                            "Err: Line {}: Non-terminating basic string.",
                            self.line_num
                        ))
                    }
                    Some(next) => {
                        seg = next;
                        continue;
                    }
                },
                Some(ch) => {
                    match ch {
                        "\"" => break,
                        "\\" => {
                            let count = seg.count();
                            match Self::process_basic_escape_sequence(ParserLine::continuation(
                                pline, count,
                            )) {
                                None => {
                                    return Err(format!(
                                        "Err: Line {}: Invalid String Escape Sequence",
                                        self.line_num
                                    ))
                                }
                                Some((ch, context)) => {
                                    pline = context;
                                    seg =
                                        {
                                            match pline.next_seg() {
                                                None => return Err(format!(
                                                    "Err: Line {}: Non-terminating basic string.",
                                                    self.line_num
                                                )),
                                                Some(next) => next,
                                            }
                                        };
                                    grapheme_pool.push(ch);
                                }
                            }
                        }
                        _ => {
                            // TODO: Add check for disallowed UTF8 characters.
                            if !is_valid_multstr_grapheme(ch) {
                                return Err(format!(
                                    "Err: Line {}: Invalid Unicode Character U+{:X}",
                                    self.line_num,
                                    ch.chars().next().unwrap() as u32,
                                ));
                            } else {
                                grapheme_pool.push_str(ch);
                            }
                        }
                    }
                }
            }
        }
        let count = seg.count();
        Ok((grapheme_pool, ParserLine::continuation(pline, count)))
    }

    pub fn process_literal_string(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(String, ParserLine), String> {
        unimplemented!()
    }

    /// Produce a UTF8 escape literal from a given iterator
    /// Assumes Unix OS so the newline can fit into a char.
    /// In the future (when I get this thing working), I can adjust for platform support
    /// which would likely require this signature to change to return &'static str OR pass
    /// the String structure in directly to push the slice to it.
    /// This function is for DRAFTING purposes ONLY. The signature will be different in the implementation.
    pub fn process_multi_escape_sequence(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(char, ParserLine, bool), String> {
        // TODO: Check logic for this function
        // Assume we have identified a backslash
        let mut seg = {
            match pline.next_seg() {
                Some(next_seg) => next_seg,
                None => return Err(String::from(EOF_ERROR)), // only condition in which a \ is followed by nothing.
            }
        };
        // from here, we *know* there is at least one character left to read.
        let outchar: char;
        let c = seg.next().unwrap();
        match c {
            "b" => outchar = '\u{0008}',
            "t" => outchar = '\t',
            "n" => outchar = '\n',
            "f" => outchar = '\u{000C}',
            "r" => outchar = '\r',
            "\"" => outchar = '\"',
            "\\" => outchar = '\\',
            "u" | "U" => match escape_utf8(&mut seg) {
                Some(c) => outchar = c,
                None => {
                    return Err(format!(
                    "Err: Line {}: Invalid UTF8 escape sequence. Format: \\uXXXX or \\uXXXXXXXX",
                    pline.line_num()
                ))
                }
            },
            _ => {
                if !c.chars().next().unwrap().is_whitespace() {
                    return Err(format!(
                        "Error: Line {}: Invalid escape sequence.",
                        pline.line_num()
                    ));
                } else {
                    // find next non-whitespace char
                    let count = seg.count();
                    match self.get_nonwhitespace(ParserLine::continuation(pline, count)) {
                        Ok((ch, context)) => {
                            if ch == '\"' {
                                return Ok((ch, context, true)); // important termination condition
                            } else {
                                return Ok((ch, context, false));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }
        let count = seg.count();
        return Ok((outchar, ParserLine::continuation(pline, count), false));
    }

    fn process_basic_escape_sequence(mut pline: ParserLine) -> Option<(char, ParserLine)> {
        // Assume we have identified a backslash
        let mut seg = {
            match pline.next_seg() {
                None => return None,
                Some(next) => next,
            }
        };

        let outchar: char;
        match seg.next() {
            Some(c) => {
                match c {
                    "b" => outchar = '\u{0008}',
                    "t" => outchar = '\t',
                    "n" => outchar = '\n',
                    "f" => outchar = '\u{000C}',
                    "r" => outchar = '\r',
                    "\"" => outchar = '\"',
                    "\\" => outchar = '\\',
                    "u" | "U" => match escape_utf8(&mut seg) {
                        None => return None,
                        Some(c) => outchar = c,
                    },
                    _ => return None, // This is where you would check for non-whitespace in a multi-string context
                }
            }
            None => return None,
        }
        let count = seg.count();
        Some((outchar, ParserLine::continuation(pline, count)))
    }

    fn get_nonwhitespace(&mut self, mut pline: ParserLine) -> Result<(char, ParserLine), String> {
        // The last line may have ended if the whitespace character was a newline, so
        // the next line is obtained in that instance.
        let mut seg = {
            match pline.next_seg() {
                Some(next_seg) => next_seg,
                None => {
                    pline = self.next_parserline()?;
                    return self.get_nonwhitespace(pline);
                }
            }
        };
        // find the non-newline
        loop {
            match seg.next() {
                Some(ch) => {
                    let ch = ch.chars().next().unwrap();
                    if !ch.is_whitespace() {
                        let count = seg.count();
                        return Ok((ch, ParserLine::continuation(pline, count)));
                    } else {
                        continue;
                    }
                }
                None => {
                    // try to get the next segment
                    match pline.next_seg() {
                        Some(next_seg) => {
                            seg = next_seg;
                            continue;
                        }
                        None => {
                            // try to get the next line
                            pline = self.next_parserline()?;
                            return self.get_nonwhitespace(pline);
                        }
                    }
                }
            }
        }
    }
}

///////////////////
// Helper Functions
///////////////////

fn is_valid_multstr_grapheme(s: &str) -> bool {
    const CHARS: [&str; 30] = [
        "\0", "\u{1}", "\u{2}", "\u{3}", "\u{4}", "\u{5}", "\u{6}", "\u{7}", "\u{8}", "\u{b}",
        "\u{c}", "\u{e}", "\u{f}", "\u{10}", "\u{11}", "\u{12}", "\u{13}", "\u{14}", "\u{15}",
        "\u{16}", "\u{17}", "\u{18}", "\u{19}", "\u{1a}", "\u{1b}", "\u{1c}", "\u{1d}", "\u{1e}",
        "\u{1f}", "\u{7f}",
    ];

    for c in CHARS {
        if s == c {
            return false;
        }
    }
    true
}
fn is_valid_str_grapheme(s: &str) -> bool {
    const CHARS: [&str; 32] = [
        "\0", "\u{1}", "\u{2}", "\u{3}", "\u{4}", "\u{5}", "\u{6}", "\u{7}", "\u{8}", "\n",
        "\u{b}", "\u{c}", "\r", "\u{e}", "\u{f}", "\u{10}", "\u{11}", "\u{12}", "\u{13}", "\u{14}",
        "\u{15}", "\u{16}", "\u{17}", "\u{18}", "\u{19}", "\u{1a}", "\u{1b}", "\u{1c}", "\u{1d}",
        "\u{1e}", "\u{1f}", "\u{7f}",
    ];

    for c in CHARS {
        if s == c {
            return false;
        }
    }
    true
}

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

fn skip_ws<'a>(iter: &mut TOMLSeg<'a>) {
    loop {
        match iter.peek() {
            Some(&ch) => match ch {
                " " | "\t" => {
                    iter.next();
                    continue;
                }
                _ => return,
            },
            None => return,
        }
    }
}

fn is_valid_comment_grapheme(s: &str) -> bool {
    is_valid_str_grapheme(s)
}

fn process_comment(mut pline: ParserLine) -> Result<(), String> {
    let line_num = pline.line_num();
    let mut iter = pline.next_seg().unwrap();
    loop {
        match iter.next() {
            Some(ch) => {
                if !is_valid_comment_grapheme(ch) {
                    return Err(format!(
                        "Invalid Comment Character: {} on Line {}",
                        ch, line_num
                    ));
                }
            }
            None => {
                let next_iter = {
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
