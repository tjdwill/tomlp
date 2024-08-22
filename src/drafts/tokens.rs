//////////
// Imports
//////////

use chrono::{offset::FixedOffset, DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;

/////////////////
// Implementation
/////////////////

pub type TOMLTable = HashMap<String, TOMLType>;

#[derive(Debug)]
/// The Rust representation of TOML value types.
pub enum TOMLType {
    Bool(bool),
    Int(i64),
    Float(f64),
    // Strings
    BasicStr(String),
    MultiStr(String),
    LitStr(String),
    MultiLitStr(String),
    // Dates
    Date(NaiveDate),
    Time(NaiveTime),
    NaiveDateTime(NaiveDateTime),
    TimeStamp(DateTime<FixedOffset>),
    // Collections
    Array(Vec<Self>),
    HTable(TOMLTable),      // Tables defined via table header syntax
    DKTable(TOMLTable),     // Tables defined via dotted keys `ex. apple.color = "red"`
    InlineTable(TOMLTable), // Needed because InlineTables are to be self-contained
    // and non-modifiable after definition
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
