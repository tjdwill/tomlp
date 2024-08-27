#![allow(dead_code)]

// Imports
use chrono::{offset::FixedOffset, DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;

// Implementation

/// Retrieve a view into the value of a given key-value pair
/// if it exists.
///
/// - `key_sequence`: the (potentially multi-part) key
/// - `delimiter`: the substring suded to separate key parts (equivalent to the `.` in a
///              dotted-key)
///
/// Arrays and Arrays of Tables cannot be queried into because they require an index.
/// It is better to retrieve the entire array structure and subsequently index them.
/// Attempting to key into an Array of Tables or an Array will result in `None`.
///
///
/// Ex.
/// ```toml
///  # sample.toml
///  [my_table]
///  key.is_dotted.example = true
/// ```
/// ```
/// use tomlp::{parse, ValFromTOMLKey, TOMLType};
/// let parsed = parse("sample.toml")?;
/// let test = parsed.retrieve("my_table\0key\0is_dotted\0example", "\0");
/// if let Some(TOMLType::Bool(b)) = test {
///     assert!(b);
/// } else {
///     panic!("This won't happen.");
/// }
/// ```
///
pub trait ValFromTOMLKey {
    fn retrieve(&self, key_sequence: &str, delimiter: &str) -> Option<&TOMLType>;
}

/// Alias for the table type.
pub type TOMLTable = HashMap<String, TOMLType>;

#[derive(Debug)]
/// The Rust representation of TOML value types.
pub enum TOMLType {
    Bool(bool),
    Int(i64),
    Float(f64),
    // Strings
    /// Basic String
    BasicStr(String),
    /// Multi-line string
    MultiStr(String),
    /// Basic literal string
    LitStr(String),
    /// Multi-line literal string
    MultiLitStr(String),
    // Dates
    Date(NaiveDate),
    Time(NaiveTime),
    NaiveDateTime(NaiveDateTime),
    TimeStamp(DateTime<FixedOffset>),
    // Collections
    Array(Vec<Self>),
    /// Table defined via table header syntax `[table]`
    HTable(TOMLTable),
    /// Table defined via dotted key (ex. `apple.color = "red"`)
    DKTable(TOMLTable),
    // Needed because InlineTables are to be self-contained and non-modifiable after definition
    InlineTable(TOMLTable),
    /// Array of Tables
    AoT(Vec<TOMLTable>), // Array of Tables
}
impl TOMLType {
    /// Gets a reference to the underlying string
    pub fn str(&self) -> Option<&str> {
        match self {
            Self::BasicStr(s) | Self::MultiStr(s) | Self::LitStr(s) | Self::MultiLitStr(s) => {
                Some(s.as_str())
            }
            _ => None,
        }
    }

    pub fn i64(&self) -> Option<i64> {
        if let Self::Int(n) = *self {
            Some(n)
        } else {
            None
        }
    }

    pub fn f64(&self) -> Option<f64> {
        if let Self::Float(n) = *self {
            Some(n)
        } else {
            None
        }
    }

    pub fn array(&self) -> Option<&Vec<Self>> {
        if let Self::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }
}

impl ValFromTOMLKey for TOMLTable {
    fn retrieve(&self, key_sequence: &str, delimiter: &str) -> Option<&TOMLType> {
        let mut curr_table = self;
        let mut curr_val: &TOMLType;
        let mut key_iter = key_sequence
            .split(delimiter)
            .map(|x| x.to_string())
            .peekable();
        let mut key: String;
        loop {
            // traverse tables
            key = key_iter.next().unwrap(); // even the empty string results in at least one
                                            // iteration.
            if let None = key_iter.peek() {
                break;
            }

            if let Some(val) = curr_table.get(&key) {
                curr_val = val;
                match curr_val {
                    TOMLType::HTable(ref table)
                    | TOMLType::DKTable(ref table)
                    | TOMLType::InlineTable(ref table) => curr_table = table,
                    _ => return None,
                }
            } else {
                // Given key not in table
                return None;
            }
        }
        if let Some(val) = curr_table.get(&key) {
            Some(val)
        } else {
            // Given key not in table
            None
        }
    }
}
