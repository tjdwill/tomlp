// stdlib imports
#![allow(unused_mut)]
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::iter::Peekable;
use std::path::Path;
// third-party imports
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

    fn next_parserline(&mut self) -> Result<ParserLine, String> {
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
        for _ in 0..3 {
            if let Some("\"") = seg.next() {
                continue;
            } else {
                return self.process_basic_string(pline);
            }
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
        for _ in 0..3 {
            seg.next();
        }
        // in multi-string context
        let mut graphemes_added = 0;
        while quote_count < 3 {
            match seg.next() {
                Some(ch) => {
                    graphemes_added += 1;
                    match ch {
                        "\"" => {
                            quote_count += 1;
                            grapheme_pool.push_str(ch);
                        }

                        "\\" => {
                            // escape sequence
                            quote_count = 0;
                            let count = seg.count();
                            let (ch, context) = self.process_multi_escape_sequence(
                                ParserLine::continuation(pline, count),
                            )?;
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
        } // found closing delimiter

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
        unimplemented!()
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
    ) -> Result<(char, ParserLine), String> {
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
            "u" | "U" => match Self::escape_utf8(&mut seg) {
                Some(c) => outchar = c,
                None => {
                    return Err(format!(
                    "Err: Line {}: Invalid UTF8 escape sequence. Format: \\uXXXX or \\uXXXXXXXX",
                    pline.line_num
                ))
                }
            },
            _ => {
                if !c.chars().next().unwrap().is_whitespace() {
                    return Err(format!(
                        "Error: Line {}: Invalid escape sequence.",
                        pline.line_num
                    ));
                } else {
                    // find next non-whitespace char
                    let count = seg.count();
                    return self.get_nonwhitespace(ParserLine::continuation(pline, count));
                }
            }
        }
        let count = seg.count();
        return Ok((outchar, ParserLine::continuation(pline, count)));
    }

    pub fn get_nonwhitespace(
        &mut self,
        mut pline: ParserLine,
    ) -> Result<(char, ParserLine), String> {
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

    pub fn escape_utf8(iter: &mut TOMLSeg<'_>) -> Option<char> {
        // try to find 4 or 8 hexadecimal digits
        const MIN_SEQ_LENGTH: i32 = 4;
        const MAX_SEQ_LENGTH: i32 = 8;

        let mut hex_val = 0_u32;
        let mut digits_processed = 0;

        while digits_processed < MAX_SEQ_LENGTH {
            if Self::is_hexdigit(iter.peek()) {
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
} // impl ParserLine

///////////////////
// Helper Functions
///////////////////

pub fn skip_ws<'a>(iter: &mut TOMLSeg<'a>) {
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

pub fn process_comment(mut pline: ParserLine) -> Result<(), String> {
    let line_num = pline.line_num;
    let mut iter = pline.next_seg().unwrap();
    loop {
        match iter.next() {
            Some(ch) => {
                if !is_valid_comment(ch) {
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

pub fn is_valid_comment(c: &str) -> bool {
    true
}
