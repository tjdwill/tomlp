#![allow(dead_code, unused_imports)]
fn main() -> Result<(), String> {
    let s = "hello".to_string();
    let vector = get_graphemes(s.as_str());
    println!("{:?}", vector);
    Ok(())
}

//////////
// Imports
//////////
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{prelude::*, BufReader};
use std::fs::File;
use std::path::Path;

use unicode_segmentation::UnicodeSegmentation as utf8;
use chrono::{offset::FixedOffset, DateTime, NaiveDate, NaiveTime};

/////////////////
// Implementation
/////////////////

fn parse(file_path: &str) -> Result<HashMap<String, TokenType>, String> {
    Ok(HashMap::new())
}


type TOMLTable = HashMap<String, TokenType>;
#[derive(Debug)]
struct TOMLParser {
    tomltable: TOMLTable,
    // TODO: decide if table names are cached.
    context: ParseContext,
}
impl TOMLParser {
    fn init(file_path: &str) -> Result<Self, String> {
        // validate the file
        let path = Path::new(file_path);
        if let Some(ext) = path.extension() {
            if ext != "toml" {
                return Err("Incorrect file extension.".to_string())
            }
            let context = ParseContext::create(path)?;
            Ok(Self { tomltable: TOMLTable::new(), context: context })
        } else {
            Err("Unknown file path error.".to_string())
        }
    }
}

#[derive(Debug)]
struct ParseContext {
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
    /// EOF: Ok(true)
    fn fill_buffer(&mut self) -> Result<bool, String> {
        self.curr_line.clear();
        self.line_num += 1;
        self.cursor = 0;
        match self.reader.read_line(&mut self.curr_line) {
            Err(err) => Err(format!("Read error on line {}: {}", self.line_num, err.kind())),
            Ok(0) => Ok(true),
            _ => Ok(false)
        }
    }

    ////////
    // Utils
    ////////
    fn get_graphemes(&self) -> Vec<&str> {
        utf8::graphemes(self.curr_line.as_str(), true).collect::<Vec<&str>>()
    }
    
    fn view_line(&self) -> &str {
        self.curr_line.as_str()
    }
}

#[derive(Debug)]
enum TokenType {}

fn get_graphemes(s: &str) -> Vec<&str> {
    utf8::graphemes(s, true).collect::<Vec<_>>()
}