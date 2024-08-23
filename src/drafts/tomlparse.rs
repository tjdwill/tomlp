// stdlib imports
#![allow(unused_mut, unused_imports, dead_code)]
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
// third-party imports
use crate::drafts::constants::{
    COMMENT_TOKEN, INLINETAB_CLOSE_TOKEN, INLINETAB_OPEN_TOKEN, KEY_VAL_SEP, SEQUENCE_DELIM, TABLE_CLOSE_TOKEN, TABLE_OPEN_TOKEN
};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use unicode_segmentation::UnicodeSegmentation;
// my imports
use super::constants::{LITERAL_STR_TOKEN, STR_TOKEN};
use super::parsetools::{ParserLine, TOMLSeg, TPath};
pub use super::tokens::{TOMLTable, TOMLType};

// Useful Types and Constants
static EOF_ERROR: &str = "End of File during parsing operation.";
type InnerParseResult<T> = Result<(T, ParserLine), String>;  // return type alias to ensure
pub struct KeyVal<'a>(pub TPath<'a>, pub TOMLType); 


#[derive(Debug)]
pub struct TOMLParser {
    buffer: String,          // Contains a given line.
    reader: BufReader<File>, // TOML File reader construct
    line_num: usize,
    table_heads: Vec<TPath<'static>>, // Contains all top-level keys of form `[key]`
    main_table: TOMLTable,            // The overall TOMLTable
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
            table_heads: Vec::new(),
            main_table: TOMLTable::new(),
        })
    }

    pub fn view_table(&self) -> &TOMLTable {
        &self.main_table
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
                }
            }
            None => return Err("Incorrect file extension.".to_string()),
        }

        match File::open(input) {
            Ok(fd) => Ok(fd),
            Err(err) => Err(format!("File Open Error: {}", err.kind())),
        }
    }

    pub fn line_num(&self) -> usize {
        self.line_num
    }
    /// returns false -> EoF
    /// Won't check for EoF mid-value parsing.
    /// I only plan to check in the outer loop.
    pub fn next_line(&mut self) -> Result<bool, String> {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => Ok(false),
            Ok(_sz) => {
                self.line_num += 1;
                Ok(true)
            }
            Err(err) => Err(format!(
                "Read error for line {1}: {0}",
                err.kind(),
                self.line_num + 1
            )),
        }
    }

    fn curr_parserline(&self) -> ParserLine {
        ParserLine::new(self.buffer.clone(), self.line_num)
    }

    pub fn next_parserline(&mut self) -> Result<ParserLine, String> {
        if !self.next_line()? {
            Err(String::from(EOF_ERROR))
        } else {
            Ok(self.curr_parserline())
        }
    }
    ////////////////////
    // Parsing Functions
    ////////////////////

    pub fn parse_keyval(&mut self, mut context: ParserLine) -> InnerParseResult<KeyVal> {
        // Assume we begin on non-whitespace
        let (key, mut context) = self.parse_key(context)?;

        let mut seg = context.next_seg().unwrap();
        // println!("EQUAL CHECK: {seg:?}");
        if seg.next() != Some(KEY_VAL_SEP) {
            return Err(format!("Line {}: Equal sign must follow key in a key-value pair.", context.line_num()));
        } else {
            seg = {
                match context.next_seg() {
                    Some(next) => next,
                    None => return Err(EOF_ERROR.to_string())
                }
            };
        }

        let count = seg.count();
        let (val, pline) = self.parse_value(ParserLine::freeze(context, count))?;
        
        Ok((KeyVal(key, val), pline))
    }
    /// Parse the input into a valid TOML type.
    pub fn parse_value(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // Assume we begin on whitespace
        let mut seg = match context.next_seg() {
            Some(next) => next,
            None => return Err(EOF_ERROR.to_string()),
        };
        seg.skip_ws();

        let ch: String;
        if let Some(&c) = seg.peek() {
            ch = c.to_string();
        } else {
            panic!("TOMLSeg should never be empty in TOMLParser::parse_value.");
        }
        let count = seg.count();
        context = ParserLine::freeze(context, count);
        match ch.as_str() {
            TABLE_OPEN_TOKEN => self.parse_array(context),
            LITERAL_STR_TOKEN => self.parse_literal_string(context),
            STR_TOKEN => self.parse_string(context),
            "t" | "f" => Self::parse_bool(context),
            INLINETAB_OPEN_TOKEN => self.parse_inline_table(context),
            _ => Self::parse_numeric(context),
        }
    }

    pub fn parse_array(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // Assume: Beginning on `[` character.
        let mut seg = context.next_seg().unwrap();
        // throw away the `[`
        seg.next();
        // get to the first inner character
        let count = seg.count();
        let (_, mut context) = self.seek_nonws(ParserLine::freeze(context, count))?;
        seg = context.next_seg().unwrap();
        
        // begin parsing
        let mut array: Vec<TOMLType> = Vec::new();
        loop {
            if let Some(&ch) = seg.peek() {
                match ch {
                    TABLE_CLOSE_TOKEN => {
                        seg.next();
                        break;
                    }
                    
                    SEQUENCE_DELIM => return Err(format!("Line {}: Array Parsing Error: The Value Separator (comma) must immediately follow a value.", context.line_num())),

                    _ => {
                        let count = seg.count();
                        let (val, pline) = self.parse_value(ParserLine::freeze(context, count))?;
                        array.push(val); 

                        // check for comma immediately following value
                        context = pline;
                        seg = context.next_seg().expect("There should still be data after a value was parsed within an array.");
                        if let Some(&SEQUENCE_DELIM) = seg.peek() {
                            seg.next();
                        }

                        // process comment, whitespace, and newline
                        let count = seg.count();
                        let (_, pline) = self.seek_nonws(ParserLine::freeze(context, count))?;
                        context = pline;
                        seg = context.next_seg().unwrap();
                    }
                }
            }
        }

        let count = seg.count();
        Ok((TOMLType::Array(array), ParserLine::freeze(context, count)))
    }

    // == String parsing ==

    /// Parses TOML-style basic and multi-line quoted strings
    pub fn parse_string(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // determine if the multi-string delimiter is present.
        // UNWRAP justification: a " character was found before calling this function, so we know the segment exists.
        let mut seg = context.peek().unwrap();
        for _ in 0..3 {
            if let Some(&STR_TOKEN) = seg.peek() {
                seg.next();
                continue;
            }
            return self.parse_basic_string(context);
        }
        self.parse_multi_string(context)
    }

    fn parse_multi_string(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let mut quote_count = 0;
        let sz = self.buffer.capacity();
        let mut grapheme_pool = String::with_capacity(sz);

        let mut seg = context.next_seg().unwrap();
        // throw away first three characters (the delimiter)
        for _ in 0..3 {
            seg.next();
        }
        // trim immediate newline if present
        if let Some(&"\n") = seg.peek() {
            seg.next();
        }
        // entered multi-string context
        let mut graphemes_added = 0;
        // terminate when we've found the three consecutive quotation marks
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
                            let (ch, pline, inc_delim) = self
                                .parse_multi_escape_sequence(ParserLine::freeze(context, count))?;

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
        } // Possibly found closing delimiter

        // Check for extra quotation mark (this is a really annoying thing to allow)
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

    fn parse_basic_string(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
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
                Some(ch) => match ch {
                    "\"" => break,
                    "\\" => {
                        let count = seg.count();
                        match Self::parse_basic_escape_sequence(ParserLine::freeze(context, count))
                        {
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
                },
            }
        }
        let count = seg.count();
        Ok((
            TOMLType::BasicStr(grapheme_pool),
            ParserLine::freeze(context, count),
        ))
    }

    /// Produces a UTF8 escape literal from a given iterator.
    /// Assumes Unix OS so the newline can fit into a char.
    /// Returns tuple of (char, ParserLine, bool) where each
    /// is (escaped_char, context, increment_delimiter)
    /// `increment_delimiter` is a way to ensure `"` are
    /// counted properly if if's the first character after a line escape.
    /// Basically, it distinguishes directly-escaped quotation marks from
    /// a quotation mark that follows a lone backslashed line break.
    pub fn parse_multi_escape_sequence(
        &mut self,
        mut context: ParserLine,
    ) -> Result<(char, ParserLine, bool), String> {
        // PLATFORM SUPPORT: After the String is made, can we transform it such that \n -> \r\n?
        // TODO: Check logic for this function
        // Assume we have identified and consumed a backslash
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
        Ok((outchar, ParserLine::freeze(context, count), false))
    }

    fn parse_basic_escape_sequence(mut context: ParserLine) -> Option<(char, ParserLine)> {
        // Assume we have identified and consumed a backslash
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
    fn get_nonwhitespace(&mut self, mut context: ParserLine) -> InnerParseResult<char> {
        // The last line may have ended if the whitespace character was a newline, so
        // the next line is obtained in that instance.
        if context.is_exhausted() {
            context = self.next_parserline()?;
        }
        let mut seg = context.next_seg().unwrap();
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

    /// Parses TOML-style basic and multi-line literal strings
    pub fn parse_literal_string(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // UNWRAP justification: a ' character was found before calling this function, so we know the segment exists.
        let mut seg = context.peek().unwrap();
        for _ in 0..3 {
            if let Some(&LITERAL_STR_TOKEN) = seg.peek() {
                seg.next();
                continue;
            }
            return self.parse_basic_litstr(context);
        }
        self.parse_multi_litstr(context)
    }

    fn parse_basic_litstr(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let mut seg = context.next_seg().unwrap();
        seg.next(); // throw away delimiter.
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
                Some(ch) => match ch {
                    LITERAL_STR_TOKEN => break,
                    _ => {
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
                },
            }
        }
        let count = seg.count();
        Ok((
            TOMLType::LitStr(grapheme_pool),
            ParserLine::freeze(context, count),
        ))
    }

    fn parse_multi_litstr(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let mut apostrophe_count = 0;
        let sz = self.buffer.capacity();
        let mut grapheme_pool = String::with_capacity(sz);

        let mut seg = context.next_seg().unwrap();
        // throw away first three characters (the delimiter)
        for _ in 0..3 {
            seg.next();
        }
        // trim immediate newline
        if let Some(&"\n") = seg.peek() {
            seg.next();
        }
        // enter multi-string context
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
                    // get a newline if necessary
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

    /// Parse the input into either an integer, a float, or a date.
    /// We don't necessarily care which.
    pub fn parse_numeric(context: ParserLine) -> InnerParseResult<TOMLType> {
        // Append the error messages since we would lose them on each subsequent parsing
        // function call..
        let mut err_msg = String::new();
        match Self::parse_integer(context.clone()) {
            Ok(out) => Ok(out),

            Err(int_msg) => {
                err_msg.push_str(int_msg.as_str());
                err_msg.push('\n');
                match Self::parse_float(context.clone()) {
                    Ok(out) => Ok(out),

                    Err(float_msg) => {
                        err_msg.push_str(float_msg.as_str());
                        err_msg.push('\n');
                        match Self::parse_date(context.clone()) {
                            Ok(out) => Ok(out),

                            Err(date_msg) => {
                                println!(
                                    "Numeric Parsing Error printout:\n{}",
                                    err_msg + date_msg.as_str()
                                );
                                Err(format!("Line {}: Could not parse numeric type. Tried integer, float, and date parsing.", context.line_num()))
                            }
                        }
                    }
                }
            }
        }
    }

    // == Integer parsing ==
    // This function doesn't need to take a mutable reference because there is no reason to
    // modify the structure. Is that the correct thing to do?
    pub fn parse_integer(mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // Assume we know some character data exists.
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        seg.peek();
        seg.skip_ws();
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
                                    seg = context.next_seg().unwrap_or_default();
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
                        (output, context) = Self::dec_parse(ParserLine::freeze(context, count))?;
                        seg = context.next_seg().unwrap_or_default();
                    }
                    _ => {
                        return Err(format!(
                            "Line {}: Invalid integer starting character '{}'.",
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

    fn dec_parse(mut context: ParserLine) -> InnerParseResult<i64> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        let mut found_underscore = false;
        let mut output: i64 = 0;

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
                                    "Line {}: Integer Parsing Error: Underscore at end of integer.",
                                    line_num
                                ));
                            } else {
                                break;
                            }
                        }
                        _ => {
                            if found_underscore {
                                return Err(format!(
                                    "Line {}: Integer Parsing Error: Underscore at end of integer.",
                                    line_num
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

        let count = seg.count();
        Ok((output, ParserLine::freeze(context, count)))
    }

    // REFACTOR POT.: I could remove the three nondec functions and instead have three different const arrays of valid
    // digit &strs. The mode would then determine which array to use and which scale factor to use (16, 8, or 2). The
    // overall logic is the same for all three inner fynctions.
    fn nondec_parse(mode: String, mut context: ParserLine) -> InnerParseResult<i64> {
        // NOTE: Consider changing mode's type to char.
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

    /// Parses the input hexadecimal into a decimal value.
    fn hex_parse(mut context: ParserLine) -> InnerParseResult<i64> {
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

    fn oct_parse(mut context: ParserLine) -> InnerParseResult<i64> {
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

    fn bin_parse(mut context: ParserLine) -> InnerParseResult<i64> {
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

    /// Parses TOML-valid float into f64
    pub fn parse_float(mut context: ParserLine) -> InnerParseResult<TOMLType> {
        // Assume a non-empty context.

        let is_keepable = |x: &&str| {
            let x = *x;
            !(x == " " || x == "\t" || x == "\n" || x == "_")
        };

        // Check for basic formatting issues
        let mut format_check_iter = context.peek().unwrap().filter(is_keepable);
        if let Some(".") = format_check_iter.next() {
            return Err(format!(
                "Line {}: Float Parsing Error: Cannot begin float with decimal point `.`",
                context.line_num()
            ));
        } else {
            let mut format_check_iter = context.peek().unwrap().filter(is_keepable).peekable();
            while let Some(ch) = format_check_iter.next() {
                if ch == "." {
                    match format_check_iter.peek() {
                        Some(&"0" | &"1"| &"2" | &"3" | &"4"| &"5" | &"6" | &"7" | &"8" | &"9") => {}
                        _ => return Err(format!(
                                "Line {}: Float Parsing Error: decimal point must be followed by a digit.",
                                context.line_num()
                            )),
                    }
                }
            }
        }

        // Parse the float
        let seg = context.next_seg().unwrap();
        let result = seg.content().trim().replace("_", "").parse::<f64>();
        match result {
            Ok(val) => Ok((TOMLType::Float(val), context)),
            Err(_) => Err(format!("Line {}: Float Parsing Error.", context.line_num())),
        }
    }

    // == DateTime Parsing ==
    pub fn parse_date(mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let mut seg = context.next_seg().unwrap();
        let test_str = seg.content().trim();
        match try_naive_dtparse(test_str) {
            Some(date) => Ok((date, ParserLine::freeze(context, 0))),
            None => Err(format!(
                "Line {}: Could not parse a datetime",
                context.line_num()
            )),
        }
    }

    // == Boolean Parsing ==
    pub fn parse_bool(mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let output: Option<bool>;
        let mut seg = context.next_seg().unwrap();

        let str_check = seg.content().trim();
        output = str_check.parse::<bool>().ok();

        match output {
            Some(val) => {
                // the segment is technically exhausted, so pass zero to freeze.
                let context = ParserLine::freeze(context, 0);
                Ok((TOMLType::Bool(val), context))
            }
            None => Err(format!(
                "Line {}: Boolean Parsing Error.",
                context.line_num()
            )),
        }
    }
    // == Table Parsing == 
    pub fn parse_inline_table(&mut self, mut context: ParserLine) -> InnerParseResult<TOMLType> {
        let mut table = TOMLTable::new();
        // Assume we begin on `{`
        let mut seg = context.next_seg().unwrap();
        // Throw away delimiter
        seg.next();
        let mut trailing_comma = false;
        loop {
            seg.skip_ws();
            if let Some(&ch) = seg.peek() {
                match ch {
                    SEQUENCE_DELIM => return Err(format!("Line {}: Inline Table Parsing Error: The value separator (comma) must immediately follow a value.", context.line_num())),

                    "\n" => return Err(format!("Line {}: Newlines are prohibited within an inline table (outside of a value that allows them)", context.line_num())),
                    
                    INLINETAB_CLOSE_TOKEN => {
                        seg.next();
                        break;
                    }
                    
                    _ => {
                        let count = seg.count();
                        let (key_val, pline) = self.parse_keyval(ParserLine::freeze(context, count))?;
                        trailing_comma = false;
                        context = pline;
                        seg = context.next_seg().expect("Inline Table should never terminate prematurely.");
                        if let Err(msg) = Self::insert(key_val, &mut table) {
                            return Err(format!("Line {}: {}", context.line_num(), msg))
                        }

                        // check for comma
                        if let Some(&SEQUENCE_DELIM) = seg.peek() {
                            seg.next();
                            trailing_comma = true;
                        }
                        
                    }

                }
            } else {
                // get next segment
                seg = match context.next_seg() {
                    Some(next) => next,
                    None => return Err(EOF_ERROR.to_string())
                };
            }
        }
        if trailing_comma {
            return Err(format!("Line {}: Trailing comma prohibited in inline tables.", context.line_num()));
        } else {
            let count = seg.count();
            Ok((TOMLType::InlineTable(table), ParserLine::freeze(context, count)))
        }
    }

    pub fn insert(kv: KeyVal, table_head: &mut TOMLTable) -> Result<(), String> {
        insert_keyval(kv, table_head)
    }

    // == Key processing ==
    /// Given an identified key location, parse the key sequence. Should handle
    /// base keys, quoted keys, and dotted keys. Only concerned with producing the
    /// key sequence itself; querying the table structure and validating unique keys are done
    pub fn parse_key(
        &mut self,
        mut context: ParserLine,
    ) -> InnerParseResult<TPath<'static>> {
        // Assume we begin at some whitespace.
        // Also assume the key itself is a dotted key since it is the most general form.

        let mut key_segs: Vec<String> = Vec::new();
        let mut temp_buf = String::new(); // for parsing bare keys
        let mut seg = context.next_seg().unwrap();
        seg.skip_ws();

        /*  Try to parse a dotted key. The termination condition is when either:
         *      - A `]` is found
         *      - An `=` is found
         */
        let mut found_quoted_str = false;
        loop {
            match seg.peek() {
                None => {
                    // This means an equal sign should be the following character
                    seg = {
                        match context.next_seg() {
                            None => {
                                return Err(format!("Line {}: {}", context.line_num(), EOF_ERROR))
                            }
                            Some(new) => new,
                        }
                    }
                }
                Some(&c) => {
                    match c {
                        TABLE_CLOSE_TOKEN | KEY_VAL_SEP => {
                            if !temp_buf.is_empty() {
                                key_segs.push(temp_buf.clone());
                                temp_buf.clear();
                            }
                            break;
                        }

                        "." => {
                            seg.next();
                            if temp_buf.is_empty() && !found_quoted_str {
                                return Err(format!(
                                    "Line {}: Bare keys cannot be empty.",
                                    context.line_num()
                                ));
                            } else if found_quoted_str {
                                found_quoted_str = false;
                            } else {
                                // push the bare key to storage
                                key_segs.push(temp_buf.clone());
                                temp_buf.clear();
                            }
                        }

                        " " | "\t" => seg.skip_ws(),

                        STR_TOKEN => {
                            let count = seg.count();
                            let (result, pline) =
                                self.parse_basic_string(ParserLine::freeze(context, count))?;
                            key_segs.push(result.str().unwrap().to_string());
                            found_quoted_str = true;

                            context = pline;
                            seg = {
                                match context.next_seg() {
                                    None => return Err(format!("Line {}: Invalid format. Keys must be followed by either an equal sign or a closing square bracket.", context.line_num())),
                                    Some(new) => new
                                }
                            }
                        }

                        LITERAL_STR_TOKEN => {
                            let count = seg.count();
                            let (result, pline) =
                                self.parse_literal_string(ParserLine::freeze(context, count))?;
                            key_segs.push(result.str().unwrap().to_string());
                            found_quoted_str = true;

                            context = pline;
                            seg = {
                                match context.next_seg() {
                                    None => return Err(format!("Line {}: Invalid format. Keys must be followed by either an equal sign or a closing square bracket.", context.line_num())),
                                    Some(new) => new
                                }
                            }
                        }

                        _ => {
                            if !is_barekey_char(c) {
                                let c = c.to_string();
                                return Err(format!(
                                    "Line {}: Invalid bare key character: {}.",
                                    context.line_num(),
                                    c
                                ));
                            } else {
                                temp_buf.push_str(c);
                                seg.next();
                            }
                        }
                    }
                }
            }
        }

        // Done parsing keys
        if key_segs.is_empty() {
            return Err(format!("Line {}: No key found.", context.line_num()));
        }

        let tpath = TPath::new(key_segs, "\0").unwrap();
        let count = seg.count();
        Ok((tpath, ParserLine::freeze(context, count)))
    }

    /// Processes table headers of form `[some_key_sequence.potentially.dotted]`
    /// Parses dotted key, performs validation, and descends the table beginning
    /// from the top-level, creating super-tables as necessary.
    /// Returns a mutable reference to the deepest table referred to by the (dotted) key
    pub fn parse_table_header(
        &mut self,
        mut context: ParserLine,
    ) -> Result<&mut TOMLTable, String> {
        let line_num = context.line_num();
        let mut seg = context.next_seg().unwrap();
        // skip the first '['
        seg.next();
        // check to see if there is an array of tables first
        if let Some(&TABLE_OPEN_TOKEN) = seg.peek() {
            // Array of tables handling here
            // TODO: Decide if I should change this overall function to assume *only* table header
            // form (as in, we know it's not an AoT?)
            seg.next();
            let count = seg.count();
            return self.parse_aot_header(ParserLine::freeze(context, count));
        }

        let count = seg.count();
        let (path, pline) = self.parse_key(ParserLine::freeze(context, count))?;
        context = pline;
        seg = context.next_seg().unwrap(); // we know the parse key function exits on either
                                           // '[' or '='
        if seg.peek() != Some(&TABLE_CLOSE_TOKEN) {
            return Err(format!(
                "Line {}: Invalid Table Header; Must close with `{}`",
                line_num, TABLE_CLOSE_TOKEN
            ));
        } else {
            // iterate until end of segment
            seg.next();
            let count = seg.count();
            Self::process_eol(ParserLine::freeze(context, count))?;
        }

        // Key Path Handling
        if !self.is_unique_table_header(&path) {
            return Err(format!(
                "Line {}: Table header `{:?}` is already defined.",
                line_num, &path
            ));
        }
        /* Here, we know the path has not been used.
         * Now, we must determine if the provided path is
         * valid in terms of table accesses.
         *
         * For each super-table, check:
         *
         *  - Does the table exist (create an empty table if not)
         *  - If the key exists already, is its corresponding value a table type (error if not)
         *  - What table type is it?
         *      - HTable: Table header -> automatically valid.
         *      - DKTable: from dotted key-val definition -> Allowed, but the *last* segment must extend the DKtable as an
         *      HTable.
         *      - AoT: Array of Tables -> Allowed; last segment of the dotted key MUST extend as HTable
         *
         */

        // Iterate through the entire key path, polling the table structure beginning from the top-level.
        // Make the iterator peekable to determine when we are on the last path segment.
        let mut path_iter = path.into_iter().peekable();
        let mut curr_table: &mut TOMLTable = &mut self.main_table;
        let mut pure_key_sequence = true; // do all key segments point to an HTable?
        let mut pathseg;
        loop {
            pathseg = path_iter.next().unwrap();
            if let None = path_iter.peek() {
                break;
            }
            //println!("{:?}", &path_iter);
            // PERF: should I declare the key buffer outside of the loop and modify it instead of
            // instantiating a new one every time?
            let key = pathseg.to_string();
            if curr_table.contains_key(&key) {
                // Try to get the next table
                match curr_table.get_mut(&key).unwrap() {
                    TOMLType::HTable(ref mut htable) => {
                        curr_table = htable;
                    }
                    TOMLType::DKTable(ref mut dktable) => {
                        curr_table = dktable;
                        pure_key_sequence = false;
                    }
                    TOMLType::AoT(ref mut aotable) => {
                        curr_table = aotable.last_mut().unwrap();  // get the latest table in the array
                        pure_key_sequence = false;
                    }
                    _ => return Err(format!(
                            "Line {}: Table header error: Dotted key component does not refer to a table.", line_num
                        )),
                }
            } else {
                // create the super table
                curr_table.insert(key.clone(), TOMLType::HTable(TOMLTable::new()));
                curr_table = {
                    if let TOMLType::HTable(ref mut new_table) = curr_table.get_mut(&key).unwrap() {
                        new_table
                    } else {
                        panic!("TOMLParser::parse_table_header - mutable reference extraction from newly-created table should never fail.");
                    }
                };
            }
        }
        // Now: the path iterator is on the last portion of the key.
        // Ex:      some.dotted.key.sequence
        //                          --------  <-- we're on this part.
        let key = pathseg.to_string();
        assert_eq!(path_iter.next(), None);

        // NOTE: This is a deliberately-nested `if` instead of an `&&` boolean.
        // The two methods are not equivalent in this context.
        if curr_table.contains_key(&key) {
            // Check for whether a non-HTable was found in the chain.
            // This portion satisfies the following excerpt from the TOML spec:
            /*
                Since tables cannot be defined more than once, redefining such tables using a [table] header is not allowed.
            */
            if !pure_key_sequence {
                return Err(format!(
                    "Line {}: Cannot redefine previously-defined table entry.",
                    line_num
                ));
            } else { // we've already checked if the entire sequence has been defined
            }
        } else {
            /*  Source: https://toml.io/en/v1.0.0#table
                The [table] form can, however, be used to define sub-tables within tables defined via dotted keys.

                ```toml
                [fruit]
                apple.color = "red"
                apple.taste.sweet = true

                # [fruit.apple]  # INVALID
                # [fruit.apple.taste]  # INVALID

                [fruit.apple.texture]  # you can add sub-tables
                smooth = true
                ```
            */
            curr_table.insert(key.clone(), TOMLType::HTable(TOMLTable::new()));
        }
        // Update the current_table reference variable
        if let TOMLType::HTable(ref mut table) = curr_table.get_mut(&key).unwrap() {
            curr_table = table;
        } else {
            return Err(format!("Line {}: Table Header Error; The last key segment must point to a table previously created as a supertable in a dotted header, or the segment must extend a table defined through either an array of tables or through a dotted key within a key-value pair.", line_num));
        }

        // Add the full path to the collection
        self.table_heads.push(path);

        // Done!
        Ok(curr_table)
    }

    fn parse_aot_header(&mut self, context: ParserLine) -> Result<&mut TOMLTable, String> {
        // assume we are *within* the square bracket delimiters already.
        let (path, mut context) = self.parse_key(context)?;
        let line_num = context.line_num();

        // == HANDLING the rest of the line ==
        let mut seg = context.next_seg().unwrap(); // we know the parse key function exits on either
                                                   // '[' or '='
                                                   // Handle closing delimiter
        if let Some(&TABLE_CLOSE_TOKEN) = seg.peek() {
            seg.next();
            if seg.peek() != Some(&TABLE_CLOSE_TOKEN) {
                return Err(format!(
                    "Line {}: Invalid Array of Tables Declaration; Must close with `{}{}`",
                    line_num, TABLE_CLOSE_TOKEN, TABLE_CLOSE_TOKEN
                ));
            } else {
                seg.next();
            }
        } else {
            return Err(format!(
                "Line {}: Invalid Array of Tables Declaration; Must close with `{}{}`",
                line_num, TABLE_CLOSE_TOKEN, TABLE_CLOSE_TOKEN
            ));
        }
        let count = seg.count();
        Self::process_eol(ParserLine::freeze(context, count))?;

        // == Validating the AoT header ==
        /*
            I don't think the TOML specification explicitly addresses if an AoT can be defined as an extension of a previously-defined table.
            As a result, I will decide. The answer is no. To nest an array of tables, the parent element must itself be an array of tables.

            In other words, in a dotted AoT key, each segment must point to an AoT if the key segment already has an associated value.
        */
        let mut curr_table: &mut TOMLTable = &mut self.main_table;
        let mut path_iter = path.into_iter().peekable();
        let mut pathseg: &str;
        loop {
            pathseg = path_iter.next().unwrap();
            if let None = path_iter.peek() {
                break;
            }
            let key = pathseg.to_string();
            if !curr_table.contains_key(&key) {
                curr_table.insert(key.clone(), TOMLType::AoT(vec![TOMLTable::new()]));
                if let TOMLType::AoT(ref mut aot) = curr_table.get_mut(&key).unwrap() {
                    // get the latest table in the array
                    curr_table = aot.last_mut().unwrap();
                } else {
                    panic!("TOMLParser::parse_aot_header should never fail to retrieve mutable reference to newly-created AoT table.");
                }
            } else if let TOMLType::AoT(ref mut aot) = curr_table.get_mut(&key).unwrap() {
                // update reference to table
                curr_table = aot.last_mut().unwrap();
            } else {
                return Err(format!("Line {}: Nested Arrays of Tables require each parent itself in the dotted key to point to an Array of Tables.", line_num));
            }
        }
        // on last segment of key
        let key = pathseg.to_string();
        if !curr_table.contains_key(&key) {
            curr_table.insert(key.clone(), TOMLType::AoT(vec![TOMLTable::new()]));
            if let TOMLType::AoT(ref mut aot) = curr_table.get_mut(&key).unwrap() {
                // get the latest table in the array
                curr_table = aot.last_mut().unwrap();
            } else {
                panic!("TOMLParser::parse_aot_header should never fail to retrieve mutable reference to newly-created AoT table.");
            }
        } else if let TOMLType::AoT(ref mut aot) = curr_table.get_mut(&key).unwrap() {
            // Insert a new table at the end of the array
            // Return the reference to said table
            aot.push(TOMLTable::new());
            curr_table = aot.last_mut().unwrap();
        } else {
            return Err(format!("Line {}: Nested Arrays of Tables require each parent itself in the dotted key to point to an Array of Tables.", line_num));
        }

        Ok(curr_table)
    }

    /// Determines if the given key path has already been used.
    fn is_unique_table_header(&self, path: &TPath<'_>) -> bool {
        let mut answer = true;
        for kp in &self.table_heads {
            if path == kp {
                answer = false;
            }
        }
        answer
    }

    // === Comment Parsing ===
    #[inline]
    fn process_comment(mut context: ParserLine,) -> InnerParseResult<()> {
        // assume beginning immediately after `#`
        let line_num = context.line_num();
        let mut seg = {
            match context.next_seg() {
                Some(next) => next,
                None => panic!("Comment parse context should never be entered with an empty line segment."),
            }
        };
        
        loop {
            match seg.next() {
                // drop the newline
                Some("\n") => {}
                Some(ch) => {
                    if !is_valid_comment_grapheme(ch) {
                        return Err(format!(
                            "Invalid Comment Character: '{}' on Line {}",
                            ch, line_num
                        ));
                    }
                }
                None => {
                    if let Some(next_iter) = context.next_seg() {
                        seg = next_iter;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(((), context))
    }
    /// Processes the end of the given line
    fn process_eol(mut context: ParserLine) -> Result<(), String> {
        let line_num = context.line_num();
        let mut iter = context.next_seg().unwrap();
        iter.skip_ws();

        match iter.next() {
            Some("\n") => {}
            Some(COMMENT_TOKEN) => {
               let count = iter.count();
               let(_, pline) = Self::process_comment(ParserLine::freeze(context, count))?;
               context = pline;
            }
            Some(ch) => {
                let ch = ch.to_string();
                return Err(format!(
                    "Line {}: Rogue non-whitespace character `{}`  (outside of comment).",
                    line_num, ch
                ));
            }
            None => {
                if let Some(seg) = context.next_seg() {
                    let count = seg.count();
                    return Self::process_eol(ParserLine::freeze(context, count));
                }
            }
        }
        assert!(context.is_exhausted());
        Ok(())
    }

    /// Handles whitespace, comments, and blank lines until a 
    /// non-whitespace/non-comment character is found
    fn seek_nonws(&mut self, mut context: ParserLine) -> InnerParseResult<()> {
        /* 
         * I can't make any assumption about the state of a given ParserLine when this function is
         *  called. We could be:
         *   - At the end of a line
         *   - At the end of the *file*
         *   - On whitespace
         *   - On non-ws
        */ 

        // Initial State
        let mut seg = {
            match context.next_seg() {
                Some(next) => next,
                None => {
                   context = self.next_parserline()?;
                   context.next_seg().unwrap()
                }
            }
        }; // Now here, we are guaranteed to have some valid segment.
        
        loop {
            // Parse until we find a non-whitespace character that is
            // neither a comment delimiter nor a newline character
            seg.skip_ws();
            if let Some(&ch) = seg.peek() {
                match ch {
                   "\n" => {seg.next();}
                   COMMENT_TOKEN => {
                       seg.next();
                       let count = seg.count();
                       let (_, pline) = Self::process_comment(ParserLine::freeze(context, count))?;
                       return self.seek_nonws(pline);
                   }
                   _ => {  // found some grapheme of interest
                     break;
                   }
                }
            } else {
                // try to get the next segment
               return self.seek_nonws(context)
            }
        }

        let count = seg.count();
        Ok(
            ((), ParserLine::freeze(context, count))
        )
    }
}

///////////////////
// Helper Functions
///////////////////

// Table Traversal

/// Given a mutable table reference, insert the provided key-value pair into the 
/// structure, defining super tables as needed.
/// Supertables are typed as dotted key tables (TOMLType::DKTable). This is the case
/// whether the dotted key is within an inline table or a higher-level structure.
fn insert_keyval(kv: KeyVal<'_>, table_head: &mut TOMLTable) -> Result<(), String> {
    let KeyVal(key_path, val) = kv;
    let mut key_iter = key_path.into_iter().peekable();
    let mut keyseg: &str;
    let mut curr_table = table_head;
    let mut partial_key: String = String::new();  // tracks the key segments that have been
                                                  // considered
    // Iterate through the preceding key segments,
    // creating DKTables as needed, and updating the table
    // pointer.
    loop {
        keyseg = key_iter.next().unwrap();
        if let None = key_iter.peek() {
            break;
        }
        partial_key.push_str(keyseg);
        partial_key.push('|');
        let key = keyseg.to_string();
        if curr_table.contains_key(&key) {
            if let TOMLType::DKTable(ref mut dktable) = curr_table.get_mut(&key).unwrap() {
                curr_table = dktable;
            } else {
                return Err(format!("Key `{}` is already defined at this table level.", partial_key))
            }
        } else {
            // create a dotted key table
            curr_table.insert(key.clone(), TOMLType::DKTable(TOMLTable::new()));
            // update curr_table
            match curr_table.get_mut(&key).unwrap() {
                TOMLType::DKTable(ref mut dktable) => curr_table = dktable,
                _ => panic!("`insert_keyval` should never reach this point. The table was just defined.")
                
            }
        }
    }
    // insert the value
    partial_key.push_str(keyseg);
    let key = keyseg.to_string();
    if curr_table.contains_key(&key) {
        return Err(format!("Key `{}` is already defined at this table level.", partial_key))
    } else {
        curr_table.insert(key, val); 
        Ok(())
    }
}

// Dates

fn try_naive_dtparse(s: &str) -> Option<TOMLType> {
    if let Ok(val) = DateTime::parse_from_rfc3339(s) {
        Some(TOMLType::TimeStamp(val))
    } else if let Some(val) = try_naive_datetime(s) {
        Some(TOMLType::NaiveDateTime(val))
    } else if let Some(val) = try_naive_date(s) {
        Some(TOMLType::Date(val))
    } else if let Some(val) = try_naive_time(s) {
        Some(TOMLType::Time(val))
    } else {
        None
    }
}

fn try_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    const NAIVEDATETIME_FORMATS: [&str; 4] = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S.%f",
        "%Y-%m-%dT%H:%M:%S.%f",
    ];

    for format in NAIVEDATETIME_FORMATS {
        if let Ok(val) = NaiveDateTime::parse_from_str(s, format) {
            return Some(val);
        }
    }
    None
}

fn try_naive_date(s: &str) -> Option<NaiveDate> {
    const NAIVEDATE_FORMAT: &str = "%Y-%m-%d";

    match NaiveDate::parse_from_str(s, NAIVEDATE_FORMAT) {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

fn try_naive_time(s: &str) -> Option<NaiveTime> {
    const NAIVETIME_FORMATS: [&str; 2] = ["%H:%M:%S.%f", "%H:%M:%S"];

    for format in NAIVETIME_FORMATS {
        if let Ok(val) = NaiveTime::parse_from_str(s, format) {
            return Some(val);
        }
    }
    None
}

// Grapheme things
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
/// Determines if the provided string is a valid bare key character.
fn is_barekey_char(s: &str) -> bool {
    match s.chars().next() {
        None => false,
        Some(c) => match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' => true,
            _ => false,
        },
    }
}

/// Produces a character from a sequence of four or eight hexadecimal digits
/// Ex. when called on the sequence `0001f525`, the output is 🔥.
fn escape_utf8(iter: &mut TOMLSeg<'_>) -> Option<char> {
    // try to find either 4 or 8 hexadecimal digits
    const SMALL_SEQ_LENGTH: i32 = 4;
    const LARGE_SEQ_LENGTH: i32 = 8;

    let mut hex_val = 0_u32;
    let mut digits_parsed = 0;

    while digits_parsed < LARGE_SEQ_LENGTH {
        if is_hexdigit(iter.peek()) {
            let digit = iter.next().unwrap();
            hex_val = 16 * hex_val + u32::from_str_radix(digit, 16).unwrap(); // UNWRAP Justification: to reach this operation, the character must be a Hex digit.
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
                Some('0'..='9' | 'A'..='F' | 'a'..='f') => true,
                _ => false, // was the empty string
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
        false
    } else {
        is_numeric(s)
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
