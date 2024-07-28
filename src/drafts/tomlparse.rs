// stdlib imports
#![allow(unused_mut)]
use chrono::format;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::iter::Peekable;
use std::path::Path;
use std::slice::RSplit;
// third-party imports
use unicode_segmentation::UnicodeSegmentation;

// my imports
use super::constants::{LITERAL_STR_TOKEN, STR_TOKEN};
use super::parsetools::{ParserLine, TOMLSeg};
pub use super::tokens::{TOMLTable, TOMLType};

static EOF_ERROR: &str = "End of File during parsing operation.";

#[derive(Debug)]
pub struct TOMLParser {
    buffer: String,
    reader: BufReader<File>,
    line_num: usize,
}
impl TOMLParser {
    // DEV Note: I think most functions will have to be made public for testing purposes.
    // Since I don't actually want a user to be able to use the functions, this will likely be a
    // private module that is then imported for a stand-alone public `parse` function.
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

    // String parsing

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

    pub fn parse_literal_string(
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
            ParserLine::freeze(context, count),
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
        let context = ParserLine::freeze(context, count);
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
                                ParserLine::freeze(context, count),
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
        let context = ParserLine::freeze(context, count);
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
                            match Self::parse_basic_escape_sequence(ParserLine::freeze(
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
            ParserLine::freeze(context, count),
        ))
    }

    /// Produce a UTF8 escape literal from a given iterator
    /// Assumes Unix OS so the newline can fit into a char.
    /// PLATFORM SUPPORT: After the String is made, can we transform it such that \n -> \r\n?
    pub fn parse_multi_escape_sequence(
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
                    match self.get_nonwhitespace(ParserLine::freeze(context, count)) {
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
        return Ok((outchar, ParserLine::freeze(context, count), false));
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
        Some((outchar, ParserLine::freeze(context, count)))
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
                        return Ok((ch, ParserLine::freeze(context, count)));
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
    pub fn parse_integer(mut context: ParserLine) -> Result<(TOMLType, ParserLine), String> {
        // Assume we know some character data exists.
        context = skip_ws(context);
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut is_negative = false;
        let mut plus_found = false;

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
        match seg.peek() {
            Some(ch) => {
                match *ch {
                    "0" => {
                        seg.next();
                        // Try parsing non-decimal format
                        match seg.next() {
                            None => (),
                            Some(prefix) => match prefix {
                                " " | "\t" | "\n" => (),

                                "b" | "o" | "x" => {
                                    if is_negative || plus_found {
                                        return Err(format!(
                                                "Line {}: Integer Parsing Error: Invalid Prefix. Write '0[box]'", line_num
                                            ));
                                    }

                                    let count = seg.count();
                                    (output, context) = Self::nondec_parse(
                                        prefix.to_string(),
                                        ParserLine::freeze(context, count),
                                    )?;
                                    seg = {
                                        match context.next_seg() {
                                            None => ParserLine::empty_iter(),
                                            Some(next) => next,
                                        }
                                    }
                                }

                                _ => {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: No leading zeros.",
                                        line_num
                                    ))
                                }
                            },
                        }
                    }
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                        let count = seg.count();
                        (output, context) =
                            Self::dec_parse(ParserLine::freeze(context, count))?;
                        seg = {
                            match context.next_seg() {
                                None => ParserLine::empty_iter(),
                                Some(next) => next,
                            }
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Line {}: Invalid integer starting character {}.",
                            line_num, ch
                        ))
                    }
                }
            }
            None => return Err(format!("Line {}: Invalid integer format.", line_num)),
        }

        if is_negative {
            output = -output;
        }
        let count = seg.count();
        context = ParserLine::freeze(context, count);
        Ok((TOMLType::Int(output), context))
    }

    fn dec_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut found_underscore = false;
        // Check for leading zero
        /*
           A leading zero is one in which a zero is followed by any other digit or non-whitespace char.
        */
        let mut output: i64 = 0;
        if let Some(&"0") = seg.peek() {
            seg.next();
            match seg.peek() {
                Some(ch) => match *ch {
                    " " | "\t" | "\n" => (),
                    _ => {}
                },
                None => (),
            }
        } else {
            // parse the number, checking for overflow
            loop {
                let previous = output;
                match seg.peek() {
                    None => {
                        if found_underscore {
                            return Err(format!(
                                "Line {}: Integer Parsing Error: Underscore at end of integer.",
                                line_num
                            ));
                        } else {
                            break;
                        }
                    }
                    Some(ch) => {
                        match *ch {
                            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                                found_underscore = false; // is overwriting faster than doing a check every iteration?
                                output = output
                                    .wrapping_mul(10)
                                    .wrapping_add(ch.parse::<i64>().unwrap());

                                if previous > output {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: Integer Overflow",
                                        line_num
                                    ));
                                }
                            }
                            "_" => {
                                if found_underscore {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: Underscore must be sandwiched between two digits (ex. `12_000`)", line_num
                                    ));
                                } else {
                                    found_underscore = true;
                                }
                            }
                            " " | "\t" | "\n" => {
                                if found_underscore {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: Underscore at end of integer.", line_num
                                    ));
                                } else {
                                    break;
                                }
                            }
                            _ => {
                                if found_underscore {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: Underscore at end of integer.", line_num
                                    ));
                                } else {
                                    return Err(format!(
                                        "Line {}: Integer Parsing Error: Invalid digit value '{}'.",
                                        line_num, ch
                                    ));
                                }
                            }
                        }
                        // advance iterator
                        seg.next();
                    }
                }
            }
        }

        let count = seg.count();
        Ok((output, ParserLine::freeze(context, count)))
    }

    // REFACTOR POT.: I could remove the three nondec functions and instead have three different const arrays of valid
    // digit &strs. The mode would then determine which array to use and which scale factor to use (16, 8, or 2). The
    // overall logic is the same for all three inner fynctions.
    fn nondec_parse(mode: String, mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        let mut seg = context.peek().unwrap();
        // preliminary check to see if the next value is some numeric.
        if !is_hexdigit(seg.peek()) {
            return Err(format!(
                "Line {}: Integer Parsing Error: Invalid integer format.",
                context.line_num()
            ));
        }
        match mode.as_str() {
            "x" => Self::hex_parse(context),
            "o" => Self::oct_parse(context),
            "b" => Self::bin_parse(context),
            _ => Err("`TOMLParser::nondec_parse` should never get here internally.".to_string()),
        }
    }

    fn hex_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut found_underscore = false;
        // parse the number, checking for overflow
        let mut output: i64 = 0;
        loop {
            let previous = output;
            match seg.peek() {
                None => break,
                Some(ch) => {
                    match *ch {
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b"
                        | "c" | "d" | "e" | "f" | "A" | "B" | "C" | "D" | "E" | "F" => {
                            let digit = i64::from_str_radix(ch, 16).unwrap();
                            found_underscore = false; // is overwriting faster than doing a check every iteration?
                            output = output.wrapping_mul(16).wrapping_add(digit);

                            if previous > output {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Integer Overflow",
                                    line_num
                                ));
                            }
                        }
                        "_" => {
                            if found_underscore {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Underscore must be sandwiched between two digits (ex. `12_000`)", line_num
                                ));
                            } else {
                                found_underscore = true;
                            }
                        }
                        " " | "\t" | "\n" => break, // NOTE: Don't need to check for comment symbol '#' because it would appear in a separate line segment.
                        _ => {
                            return Err(format!(
                                "Line {}: Integer Parsing Error: Invalid digit value '{}'.",
                                line_num, ch
                            ))
                        }
                    }
                    // advance iterator
                    seg.next();
                }
            }
        }

        let count = seg.count();
        Ok((output, ParserLine::freeze(context, count)))
    }

    fn oct_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut found_underscore = false;
        // parse the number, checking for overflow
        let mut output: i64 = 0;
        loop {
            let previous = output;
            match seg.peek() {
                None => break,
                Some(ch) => {
                    match *ch {
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" => {
                            found_underscore = false; // is overwriting faster than doing a check every iteration?
                            output = output
                                .wrapping_mul(8)
                                .wrapping_add(ch.parse::<i64>().unwrap());

                            if previous > output {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Integer Overflow",
                                    line_num
                                ));
                            }
                        }
                        "_" => {
                            if found_underscore {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Underscore must be sandwiched between two digits (ex. `12_000`)", line_num
                                ));
                            } else {
                                found_underscore = true;
                            }
                        }
                        " " | "\t" | "\n" => break, // NOTE: Don't need to check for comment symbol '#' because it would appear in a separate line segment.
                        _ => {
                            return Err(format!(
                                "Line {}: Integer Parsing Error: Invalid digit value '{}'.",
                                line_num, ch
                            ))
                        }
                    }
                    // advance iterator
                    seg.next();
                }
            }
        }

        let count = seg.count();
        Ok((output, ParserLine::freeze(context, count)))
    }

    fn bin_parse(mut context: ParserLine) -> Result<(i64, ParserLine), String> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut found_underscore = false;
        // parse the number, checking for overflow
        let mut output: i64 = 0;
        loop {
            let previous = output;
            match seg.peek() {
                None => break,
                Some(ch) => {
                    match *ch {
                        "0" | "1" => {
                            found_underscore = false; // is overwriting faster than doing a check every iteration?
                            output = output
                                .wrapping_mul(2)
                                .wrapping_add(ch.parse::<i64>().unwrap());

                            if previous > output {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Integer Overflow",
                                    line_num
                                ));
                            }
                        }
                        "_" => {
                            if found_underscore {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Underscore must be sandwiched between two digits (ex. `12_000`)", line_num
                                ));
                            } else {
                                found_underscore = true;
                            }
                        }
                        " " | "\t" | "\n" => break, // NOTE: Don't need to check for comment symbol '#' because it would appear in a separate line segment.
                        _ => {
                            return Err(format!(
                                "Line {}: Integer Parsing Error: Invalid digit value '{}'.",
                                line_num, ch
                            ))
                        }
                    }
                    // advance iterator
                    seg.next();
                }
            }
        }

        let count = seg.count();
        Ok((output, ParserLine::freeze(context, count)))
    }

    pub fn parse_float(mut context: ParserLine) -> Result<(TOMLType, ParserLine), String> {
        // Assume a non-empty context.
        // This isn't one-to-one with the TOML spec, but, honestly, I will accept it.
        // It beats the alternative of manually parsing IEE 754 binary64 floats.

        let removable = |x: &&str| {
            let x = *x;
            if x == " " || x == "\t" || x == "\n" || x == "_" {
                false
            } else {
                true
            }
        };

        // Check for basic formatting issues
        let mut format_check_iter = context.peek().unwrap().filter(removable);
        if let Some(".") = format_check_iter.next() {
            return Err(format!(
                "Line {}: Float Parsing Error: Cannot begin float with decimal point `.`",
                context.line_num()
            ));
        } else if let Some(".") = format_check_iter.last() {
            return Err(format!(
                "Line {}: Float Parsing Error: Cannot begin float with decimal point `.`",
                context.line_num()
            ));
        }

        let seg = context.next_seg().unwrap();
        let result = seg.filter(removable).collect::<String>().parse::<f64>();
        match result {
            Ok(val) => Ok((TOMLType::Float(val), context)),
            Err(_) => Err(format!("Line {}: Float Parsing Error.", context.line_num())),
        }
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
    const SMALL_SEQ_LENGTH: i32 = 4;
    const LARGE_SEQ_LENGTH: i32 = 8;

    let mut hex_val = 0_u32;
    let mut digits_parsed = 0;

    while digits_parsed < LARGE_SEQ_LENGTH {
        if is_hexdigit(iter.peek()) {
            let digit = iter.next().unwrap();
            hex_val = 16 * hex_val + u32::from_str_radix(digit, 16).unwrap();
        } else if digits_parsed == SMALL_SEQ_LENGTH {
            break;
        } else {
            return None;
        }
        digits_parsed += 1;
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

fn is_numeric(s: &str) -> bool {
    match s {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => true,
        _ => false,
    }
}

fn is_octal(s: &str) -> bool {
    if s == "8" || s == "9" {
        return false;
    } else {
        is_numeric(s)
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
    ParserLine::freeze(context, count)
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

#[cfg(test)]
mod tests {
    use super::*;

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
