//////////
// Imports
//////////

use chrono::{offset::FixedOffset, DateTime, NaiveDate, NaiveTime};
use std::collections::HashMap;
use std::hash::Hash;

/////////////////
// Implementation
/////////////////

/*
fn parse(file_path: &str) -> Result<HashMap<String, TOMLType>, String> {
    Ok(HashMap::new())
}
*/

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
    TimeStamp(DateTime<FixedOffset>),
    // Collections
    Array(Vec<Self>),
    Table(HashMap<String, Self>),
    InlineTable(HashMap<String, Self>),
}
