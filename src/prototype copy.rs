#![allow(dead_code, unused_imports)]

//////////////
// Module Decs
//////////////

mod parserline;
mod constants;

//////////
// Imports
//////////

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{ prelude::*, BufReader };
use std::iter::{ Peekable, Skip};
use std::fs::File;
use std::path::Path;

use unicode_segmentation::{UnicodeSegmentation as utf8, Graphemes};
use chrono::{offset::FixedOffset, DateTime, NaiveDate, NaiveTime};

use constants::*;
use parserline::ParserLine;
/////////////////
// Implementation
/////////////////

/*
fn parse(file_path: &str) -> Result<HashMap<String, TOMLType>, String> {
    Ok(HashMap::new())
}
*/

type TOMLTable = HashMap<String, TOMLType>;
#[derive(Debug)]
pub struct TOMLParser {
    tomltable: TOMLTable,
    // TODO: decide if table names are cached.
    pub context: ParseContext,
}
impl TOMLParser {
    pub fn init(file_path: &str) -> Result<Self, String> {
        // validate the file
        let path = Path::new(file_path);
        if let Some(ext) = path.extension() {
            if ext != "toml" {
                return Err("Incorrect file extension.".to_string())
            }
            let context = ParseContext::create(path)?;
            Ok(Self { tomltable: TOMLTable::new(), context: context })
        } else {
            Err(format!("Could not find extension for file {:?}.", path))
        }
    }

    pub fn fill_buffer(&mut self) -> Result<bool, String> {
        self.context.fill_buffer()
    }

    /////////////////
    // Parsing Funcs 
    /////////////////
    
    pub fn parse(&mut self) -> Result<TOMLTable, String> {
        
        while self.context.fill_buffer()? {
            if self.process_blankln() {
                continue
            } else if self.process_comment()? {
                continue
            } else {} 
        }

        Ok(std::mem::replace(&mut self.tomltable, TOMLTable::new()))
    }

    /// Determine if the current top-level structure is a table declaration
    /// basic table `[some table]` or array of tables `[[some table]]`
    pub fn find_table(&mut self) -> Result<bool, String> {
        unimplemented!()
    }

    pub fn process_table(&mut self) -> Result<bool, String> {
        self.skip_leading_ws();
        unimplemented!()
    }

    pub fn process_inline_table(&mut self) -> Result<bool, String> {
        unimplemented!();
    }

    pub fn process_string(&mut self) -> Result<TOMLType, String> {
        Ok(TOMLType::BasicStr("".to_string()))
    }

    pub fn process_comment(&mut self) -> Result<bool, String> {
        self.skip_leading_ws();
        let mut graphemes = self.context.skipped_iter();
        if let Some(char_ref) = graphemes.peek() {
            if *char_ref == COMMENT_TOKEN {
                match graphemes.find(|x| invalid_comment_char(x))
                {
                    Some(item) => Err(format!(
                        "Found invalid comment input: '{}' on line {}.",
                        item, self.context.line_num
                    )),
                    None => {
                        // TODO: Replace with assigning the curr_line string's length directly.
                        // One less iteration cycle per call.
                        self.context.cursor += self.context.skipped_iter().count(); // at end of line
                        Ok(true)
                    }
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn process_blankln(&mut self) -> bool {
        let mut ret = false;
        self.skip_leading_ws();
        if self.context.skipped_iter().count() == 0 {
            ret = true;
        }
        ret
    }

    pub fn skip_leading_ws(&mut self) {
        let mut skips = 0;
        for c in self.context.skipped_iter() {
            match c {
                " " | "\t" => {skips += 1; continue}
                _ => break
            }
        }
        self.context.cursor += skips;
    }
}

#[derive(Debug)]
pub struct ParseContext {
    line_num: usize,
    cursor: usize,
    curr_line: String,
    reader: BufReader<File>,
}

impl ParseContext {
    fn create(file_path: &Path) -> Result<Self, String> {
        let file = {
            match File::open(file_path) {
                Ok(fd) => fd,
                Err(err) => return Err(format!("File open error: {}. File: {:?}", err.kind(), file_path))
            }
        };
        let reader = BufReader::new(file);
        Ok(Self {
            line_num: 0,
            cursor: 0,
            curr_line: String::with_capacity(400),  // Default to 100 four-byte UTF-8 codepoints.
            reader: reader,
        })
    }

    
    /// Fills the current line buffer
    /// Returns a boolean Result to communicate if EOF has been reached.
    /// EOF: Ok(false)
    fn fill_buffer(&mut self) -> Result<bool, String> {
        //assert!(self.cursor >= self.curr_line.len());
        self.curr_line.clear();
        self.line_num += 1;
        self.cursor = 0;
        match self.reader.read_line(&mut self.curr_line) {
            Err(err) => Err(format!("Read error on line {}: {}", self.line_num, err.kind())),
            Ok(0) => Ok(false),
            _ => {
                self.curr_line.pop();  // get rid of the newline character
                Ok(true)
            }
        }
    }

    ////////
    // Utils
    ////////
    pub fn char_iter(&self) -> Graphemes {
        utf8::graphemes(self.curr_line.as_str(), true)
    }

    pub fn skipped_iter(&self) -> Peekable<Skip<Graphemes>> {
       self.char_iter().skip(self.cursor).peekable()
    }
    
    pub fn view_line(&self) -> &str {
        self.curr_line.as_str()
    }
}

#[derive(Debug)]
pub enum TOMLType {
    // VALUES
    Bool(bool),
    Int(i64),
    Float(f64),
    // Strings
    BasicStr(String),
    MultStr(String),
    LitStr(String),
    MultLitStr(String),
    // Dates
    Date(NaiveDate),
    Time(NaiveTime),
    TimeStamp(DateTime<FixedOffset>),
    // Collections
    Array(Vec<Self>),
    Table(HashMap<String, Self>),
    InlineTable(HashMap<String, Self>),
}

///////////////
// Helper Funcs
///////////////

fn invalid_comment_char(s: &str) -> bool {
    const CHARS: [&str; 32] = [
        "\0", "\u{1}", "\u{2}", "\u{3}", "\u{4}", "\u{5}", "\u{6}", "\u{7}", "\u{8}", "\n",
        "\u{b}", "\u{c}", "\r", "\u{e}", "\u{f}", "\u{10}", "\u{11}", "\u{12}", "\u{13}",
        "\u{14}", "\u{15}", "\u{16}", "\u{17}", "\u{18}", "\u{19}", "\u{1a}", "\u{1b}",
        "\u{1c}", "\u{1d}", "\u{1e}", "\u{1f}", "\u{7f}",
    ];

    for c in CHARS {
        if s == c {
            return true;
        }
    }
    false
}
fn get_graphemes(s: &str) -> Vec<&str> {
    utf8::graphemes(s, true).collect::<Vec<_>>()
}
/// Use this to instantiate an array of invalid chars via Copy/Paste.
pub fn print_invalid_comment_chars() {
    println!("Invalid TOML Comment Char Report");
    let inval_string = get_invalid_comment_chars();
    let invalids = get_graphemes(inval_string.as_str());
    println!("Total Num of Invalids: {}", invalids.len());
    println!("Invalid Comment Chars: {:?}\n", invalids);
}

fn get_invalid_comment_chars() -> String {
    let range1 = 0_u8..=8_u8;
    let range2 =
        u8::from_str_radix("A", 16).unwrap()..=u8::from_str_radix("1F", 16).unwrap();
    let range3 =
        u8::from_str_radix("7F", 16).unwrap()..u8::from_str_radix("80", 16).unwrap();
    let chars = range1.chain(range2.chain(range3)).collect::<Vec<u8>>();
    String::from_utf8(chars).unwrap()
}

