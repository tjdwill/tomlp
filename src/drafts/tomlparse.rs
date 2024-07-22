// stdlib imports
#![allow(unused_mut)]
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::iter::Peekable;
use std::path::Path;
use std::slice::RSplit;
// third-party imports
use unicode_segmentation::UnicodeSegmentation;

// my imports
use super::constants::{LITERAL_STR_TOKEN, STR_TOKEN};
use super::parsetooling::{ParserLine, TOMLSeg};
pub use super::tokens::{TOMLTable, TOMLType};

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

    // String parseing

    pub fn parse_string(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // determine if the multi-string delimiter is present.
        // UNWRAP justification: a " character was found before calling this function, so we know the segment exists.
        let mut seg = context.peek().unwrap();
        for i in 0..3 {
            if let Some(&STR_TOKEN) = seg.peek() {
                seg.next();
                continue;
            }
            return self.parse_basic_string(context);
        }
        return self.parse_multi_string(context);
    }

    fn parse_literal_string(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // UNWRAP justification: a " character was found before calling this function, so we know the segment exists.
        let mut seg = context.peek().unwrap();
        for i in 0..3 {
            if let Some(&LITERAL_STR_TOKEN) = seg.peek() {
                seg.next();
                continue;
            }
            return self.parse_basic_litstr(context);
        }
        return self.parse_multi_litstr(context);
    }

    fn parse_basic_litstr(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // throw away delimiter
        let mut seg = context.next_seg().unwrap();
        seg.next();
        let mut grapheme_pool = String::with_capacity(self.buffer.capacity());
        loop {
            match seg.next() {
                None => match context.next_seg() {
                    None => {
                        return Err(format!(
                            "Err: Line {}: Non-terminating literal string.",
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
                        LITERAL_STR_TOKEN => break,
                        _ => {
                            // TODO: Add check for disallowed UTF8 characters.
                            if !is_valid_litstr_grapheme(ch) {
                                return Err(format!(
                                    "Err: Line {}: Invalid Unicode Character U+{:X} in literal string.",
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
        Ok((
            TOMLType::LitStr(grapheme_pool),
            ParserLine::continuation(context, count),
        ))
    }

    fn parse_multi_litstr(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // throw away first three characters (the delimiter)
        let mut apostrophe_count = 0;
        let sz = self.buffer.capacity();
        let mut grapheme_pool = String::with_capacity(sz);

        let mut seg = context.next_seg().unwrap();
        for i in 0..3 {
            seg.next();
        }
        // trim immediate newline
        if let Some(&"\n") = seg.peek() {
            seg.next();
        }
        // in multi-string context
        let mut graphemes_added = 0;
        while apostrophe_count < 3 {
            match seg.next() {
                Some(ch) => {
                    graphemes_added += 1;
                    match ch {
                        LITERAL_STR_TOKEN => apostrophe_count += 1,
                        _ => apostrophe_count = 0,
                    }
                    grapheme_pool.push_str(ch);
                }

                None => {
                    if context.is_exhausted() {
                        context = self.next_parserline()?;
                    }
                    seg = context.next_seg().unwrap();
                }
            }
        } // Found closing delimiter

        // check for extra apostrophe (this is a really annoying thing to allow)
        // REFERENCE: https://toml.io/en/v1.0.0#string
        if let Some(&LITERAL_STR_TOKEN) = seg.peek() {
            grapheme_pool.push_str(seg.next().unwrap());
            graphemes_added += 1;
        }

        let outstring = grapheme_pool
            .as_str()
            .graphemes(true)
            .take(graphemes_added - 3)
            .collect::<String>();
        let count = seg.count();
        let context = ParserLine::continuation(context, count);
        Ok((TOMLType::MultiLitStr(outstring), context))
    }

    fn parse_multi_string(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // throw away first three characters (the delimiter)
        let mut quote_count = 0;
        let sz = self.buffer.capacity();
        let mut grapheme_pool = String::with_capacity(sz);

        let mut seg = context.next_seg().unwrap();
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
                            let (ch, pline, inc_delim) = self.parse_multi_escape_sequence(
                                ParserLine::continuation(context, count),
                            )?;

                            if inc_delim {
                                quote_count += 1;
                            } else {
                                quote_count = 0;
                            }

                            grapheme_pool.push(ch);

                            context = pline;
                            if context.is_exhausted() {
                                context = self.next_parserline()?;
                            }
                            seg = context.next_seg().unwrap();
                        }

                        _ => {
                            quote_count = 0;
                            grapheme_pool.push_str(ch);
                        }
                    }
                }
                None => {
                    if context.is_exhausted() {
                        context = self.next_parserline()?;
                    }
                    seg = context.next_seg().unwrap();
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
        let context = ParserLine::continuation(context, count);
        Ok((TOMLType::MultiStr(outstring), context))
    }

    fn parse_basic_string(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(TOMLType, ParserLine), String> {
        // Throw away first character (delimiter)
        let mut seg = context.next_seg().unwrap();
        seg.next();
        let mut grapheme_pool = String::with_capacity(self.buffer.capacity());
        loop {
            match seg.next() {
                None => match context.next_seg() {
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
                            match Self::parse_basic_escape_sequence(ParserLine::continuation(
                                context, count,
                            )) {
                                None => {
                                    return Err(format!(
                                        "Err: Line {}: Invalid String Escape Sequence",
                                        self.line_num
                                    ))
                                }
                                Some((ch, pline)) => {
                                    context = pline;
                                    seg = {
                                        match context.next_seg() {
                                            None => {
                                                return Err(format!(
                                                    "Err: Line {}: Non-terminating basic string.",
                                                    self.line_num
                                                ))
                                            }
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
        Ok((
            TOMLType::BasicStr(grapheme_pool),
            ParserLine::continuation(context, count),
        ))
    }

    /// Produce a UTF8 escape literal from a given iterator
    /// Assumes Unix OS so the newline can fit into a char.
    /// PLATFORM SUPPORT: After the String is made, can we transform it such that \n -> \r\n?
    fn parse_multi_escape_sequence(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(char, ParserLine, bool), String> {
        // TODO: Check logic for this function
        // Assume we have identified a backslash
        let mut seg = {
            match context.next_seg() {
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
                    context.line_num()
                ))
                }
            },
            _ => {
                if !c.chars().next().unwrap().is_whitespace() {
                    return Err(format!(
                        "Error: Line {}: Invalid escape sequence.",
                        context.line_num()
                    ));
                } else {
                    // find next non-whitespace char
                    let count = seg.count();
                    match self.get_nonwhitespace(ParserLine::continuation(context, count)) {
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
        return Ok((outchar, ParserLine::continuation(context, count), false));
    }

    fn parse_basic_escape_sequence(mut context: ParserLine) -> Option<(char, ParserLine)> {
        // Assume we have identified a backslash
        let mut seg = {
            match context.next_seg() {
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
        Some((outchar, ParserLine::continuation(context, count)))
    }

    /// Proceeds to the next non-whitespace character in the buffer.
    /// Moves to next line if necessary.
    /// Whitespace in *this* instance is defined as Unicode whitespace; it's a superset of TOML whitespace.
    fn get_nonwhitespace(&mut self, mut context: ParserLine) -> Result<(char, ParserLine), String> {
        // The last line may have ended if the whitespace character was a newline, so
        // the next line is obtained in that instance.
        let mut seg = {
            match context.next_seg() {
                Some(next_seg) => next_seg,
                None => {
                    context = self.next_parserline()?;
                    return self.get_nonwhitespace(context);
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
                        return Ok((ch, ParserLine::continuation(context, count)));
                    } else {
                        continue;
                    }
                }
                None => {
                    // try to get the next segment
                    match context.next_seg() {
                        Some(next_seg) => {
                            seg = next_seg;
                            continue;
                        }
                        None => {
                            // try to get the next line
                            context = self.next_parserline()?;
                            return self.get_nonwhitespace(context);
                        }
                    }
                }
            }
        }
    }

    // Integer parsing
    // This function doesn't need to take a mutable reference because there is no reason to
    // modify the structure. Is that the correct thing to do?
    fn parse_integer(mut context: ParserLine) -> Result<(TOMLType, ParserLine), String> {
        // Assume we know some character data exists.
        context = skip_ws(context);
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut is_negative = false;
        let mut plus_found = false;
        let mut check_prefix = false;

        {
            let ch = *seg.peek().unwrap(); // UNWRAP Justification: the first character is always present
            match ch {
                "+" => {
                    plus_found = true;
                    seg.next();
                }
                "-" => {
                    is_negative = true;
                    seg.next();
                }
                _ => (),
            }
        }

        // Check for a prefix directive.
        let mut output: i64 = 0;
        match seg.next() {
            Some(ch) => {
                match ch {
                    "0" => {
                        check_prefix = true;
                    }
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                        let count = seg.count();
                        (output, context) =
                            Self::dec_parse(ParserLine::continuation(context, count))?;
                        seg = context.next_seg().unwrap(); // UNWRAP Justification: function exits on whitespace.
                    }
                    _ => {
                        return Err(format!(
                            "Line {}: Invalid integer character {}.",
                            line_num, ch
                        ))
                    }
                }
            }
            None => return Err(format!("Line {}: Invalid integer format.", line_num)),
        }

        // Try parsing non-decimal format.
        if check_prefix {
            match seg.next() {
                Some(prefix) => {
                    match prefix {
                        "b" | "o" | "x" => {
                            if is_negative || plus_found {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Invalid Prefix. Write '0[box]'", line_num
                                ));
                            }
                            let count = seg.count();
                            (output, context) = Self::nondec_parse(
                                prefix.to_string(),
                                ParserLine::continuation(context, count),
                            )?;
                            seg = context.next_seg().unwrap(); // UNWRAP Justification: function exits on whitespace.
                        }
                        _ => {
                            let count = seg.count();
                            (output, context) =
                                Self::dec_parse(ParserLine::continuation(context, count))?;
                            seg = context.next_seg().unwrap(); // UNWRAP Justification: function exits on whitespace.
                        }
                    }
                }
                None => (),
            }
        }

        if is_negative {
            output = -output;
        }
        let count = seg.count();
        context = ParserLine::continuation(context, count);
        Ok((TOMLType::Int(output), context))
    }

    fn dec_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        unimplemented!()
    }

    fn nondec_parse(mode: String, mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        match mode.as_str() {
            "x" => Self::hex_parse(context),
            "o" => Self::oct_parse(context),
            "b" => Self::bin_parse(context),
            _ => Err("Never gets here.".to_string()),
        }
    }

    fn hex_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        unimplemented!()
    }

    fn oct_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        unimplemented!()
    }

    fn bin_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        unimplemented!()
    }
}

///////////////////
// Helper Functions
///////////////////

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
fn is_valid_multstr_grapheme(s: &str) -> bool {
    if s == "\n" || s == "\r" {
        true
    } else {
        is_valid_str_grapheme(s)
    }
}
fn is_valid_litstr_grapheme(s: &str) -> bool {
    if s == "\t" {
        true
    } else {
        is_valid_str_grapheme(s)
    }
}
fn is_valid_multi_litstr_grapheme(s: &str) -> bool {
    if s == "\t" || s == "\n" {
        true
    } else {
        is_valid_str_grapheme(s)
    }
}
fn is_valid_comment_grapheme(s: &str) -> bool {
    is_valid_str_grapheme(s)
}

fn escape_utf8(iter: &mut TOMLSeg<'_>) -> Option<char> {
    // try to find 4 or 8 hexadecimal digits
    const MIN_SEQ_LENGTH: i32 = 4;
    const MAX_SEQ_LENGTH: i32 = 8;

    let mut hex_val = 0_u32;
    let mut digits_parseed = 0;

    while digits_parseed < MAX_SEQ_LENGTH {
        if is_hexdigit(iter.peek()) {
            let digit = iter.next().unwrap();
            hex_val = 16 * hex_val + u32::from_str_radix(digit, 16).unwrap();
        } else if digits_parseed == MIN_SEQ_LENGTH {
            break;
        } else {
            return None;
        }
        digits_parseed += 1;
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

/// skip the whitespace at the current segment
fn skip_ws(mut context: ParserLine) -> ParserLine {
    let mut seg = {
        match context.next_seg() {
            None => return context,
            Some(next) => next,
        }
    };

    loop {
        match seg.peek() {
            Some(&ch) => match ch {
                " " | "\t" => {
                    seg.next();
                }
                _ => break,
            },
            None => break,
        }
    }
    let count = seg.count();
    ParserLine::continuation(context, count)
}

fn parse_comment(mut context: ParserLine) -> Result<(), String> {
    let line_num = context.line_num();
    let mut iter = context.next_seg().unwrap();
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
                    match context.peek() {
                        Some(iter) => iter,
                        None => return Ok(()),
                    }
                };
                return parse_comment(context);
            }
        }
    }
}
