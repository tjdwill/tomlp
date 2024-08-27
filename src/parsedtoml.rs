#![allow(unused_variables, unused_imports)]
use crate::tomltypes::TOMLType;

use super::parsetools::TPath;
use super::tomlparse::TOMLParser;
use super::tomltypes::TOMLTable;
/// The Rust representation of the complete read-only TOML table.
/// It's a wrapped hash map.
/// Goals:
///     - [ ] Create a printable representation reminiscent to Unix's `tree`
///     - [ ] Allow queries using TOML keys
#[derive(Debug)]
pub struct ParsedTOML {
    table: TOMLTable,
}
impl ParsedTOML {
    pub(super) fn from(table: TOMLTable) -> Self {
        Self { table }
    }

    /// Retrieve a view into the value of a given key-value pair
    /// if it exists.
    pub fn get(key: TPath<'_>) -> Option<&TOMLType> {
        todo!();
    }

    /// A function for recursively descending and printing the TOML table.
    /// Takes inspiration from the `tree` program.
    fn tree(&self) -> String {
        let mut last_key_tracker: Vec<bool> = Vec::new();
        Self::tree_iter(&self.table, 0, "/".to_string(), &mut last_key_tracker)
    }
    fn tree_iter(table: &TOMLTable, level: usize, mut outstr: String, last_key_tracker: &mut Vec<bool>) -> String {
        /*
         * - For the given table, iterate over all keys. 
         *
         */
        const TERMINATING_CONNECTOR: &str = "└── ";
        const NONTERMINATING_CONNECTOR: &str = "├── ";
        const VERTICAL_EXTENDER: &str = "│";
        const SPACING: &str = "   "; // three spaces

        last_key_tracker.push(false);

        let mut key_iter = table.keys().peekable();
        let mut connector: &str = NONTERMINATING_CONNECTOR;
        while let Some(key) = key_iter.next() {
            if let None = key_iter.peek() {
                connector = TERMINATING_CONNECTOR;
                last_key_tracker[level] = true;
            }
            outstr.push('\n');
            // print the key
            for lv in 0..level {
                let is_last_key = *last_key_tracker.get(lv).unwrap();
                if is_last_key {
                    outstr.push_str(" ");
                } else {
                    outstr.push_str(VERTICAL_EXTENDER);
                }
                outstr.push_str(SPACING);
            }
            outstr.push_str(connector);
            outstr.push_str(key.as_str());
            // handle the value
            // First, check for recursive table
            let toml_val = table.get(key).unwrap();
            if let TOMLType::HTable(ref htable) = toml_val {
                outstr = Self::tree_iter(htable, level+1, outstr, last_key_tracker);
                continue;
            } else if let TOMLType::DKTable(ref dktable) = toml_val {
                outstr = Self::tree_iter(dktable, level+1, outstr, last_key_tracker);
                continue;
            } else if let TOMLType::InlineTable(ref inlinetab) = toml_val {
                outstr = Self::tree_iter(inlinetab, level+1, outstr, last_key_tracker);
                continue;
            } else if let TOMLType::AoT(ref aot) = toml_val {
                outstr.push_str(" (Arr_of_Tbls)"); 
                for table in aot.iter() {
                    outstr = Self::tree_iter(table, level+1, outstr, last_key_tracker);
                }
                continue;
            }
            // If we reach here, there's a value that we're just labeling instead of expanding.
            outstr.push('\n');
             for lv in 0..level+1 {
                let is_last_key = *last_key_tracker.get(lv).unwrap();
                if is_last_key {
                    outstr.push_str(" ");
                } else {
                    outstr.push_str(VERTICAL_EXTENDER);
                }
                outstr.push_str(SPACING);
            }
            outstr.push_str(TERMINATING_CONNECTOR);
            match toml_val {
                TOMLType::Array(_) => outstr.push_str("ARRAY"),
                TOMLType::MultiStr(_s) => outstr.push_str("MULTI-LINE STRING"),
                TOMLType::MultiLitStr(_) => outstr.push_str("MULTI-LINE LITERAL STRING"),
                TOMLType::LitStr(s) => outstr.push_str(s),
                TOMLType::BasicStr(s) => outstr.push_str(s),
                TOMLType::Bool(bl) => outstr.push_str(bl.to_string().as_str()),
                TOMLType::Int(i) => outstr.push_str(i.to_string().as_str()),
                TOMLType::Float(f) => outstr.push_str(f.to_string().as_str()),
                TOMLType::TimeStamp(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::NaiveDateTime(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::Date(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::Time(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::HTable(_) | TOMLType::DKTable(_) | TOMLType::InlineTable(_) | TOMLType::AoT(_) => (),
            }
        }
        last_key_tracker.pop();  // remove this level's boolean
        assert_eq!(last_key_tracker.len(), level);
        outstr
    }
}
impl std::fmt::Display for ParsedTOML {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree())
    }
}

