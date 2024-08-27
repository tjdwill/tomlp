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
        Self::tree_iter(&self.table, 1, ".".to_string())
    }
    fn tree_iter(table: &TOMLTable, level: usize, mut outstr: String) -> String {
        /*
         * - For the given table, iterate over all keys. 
         *
         */
        if table.is_empty() {
            return "".to_string();
        }

        const TERMINATING_CONNECTOR: &str = "└── ";
        const NONTERMINATING_CONNECTOR: &str = "├── ";
        const VERTICAL_EXTENDER: &str = "│";
        const SPACING: &str = "   "; // three spaces

        let mut key_iter = table.keys().peekable();
        let mut connector: &str = NONTERMINATING_CONNECTOR;
        while let Some(key) = key_iter.next() {
            if let None = key_iter.peek() {
                connector = TERMINATING_CONNECTOR;
            }
            outstr.push('\n');
            // print the key
            if level > 1 {
                for _ in 0..level-1 {
                    outstr = outstr + VERTICAL_EXTENDER + SPACING;
                }
            }
            outstr.push_str(connector);
            outstr.push_str(key.as_str());
            // handle the value
            // First, check for recursive table
            let toml_val = table.get(key).unwrap();
            if let TOMLType::HTable(ref htable) = toml_val {
                outstr = Self::tree_iter(htable, level+1, outstr);
            } else if let TOMLType::DKTable(ref dktable) = toml_val {
                outstr = Self::tree_iter(dktable, level+1, outstr)
            }
            // If we reach here, there's a value that we're just labeling instead of expanding.
            outstr.push('\n');
            if level > 1 {
                for _ in 0..level {
                    outstr = outstr + VERTICAL_EXTENDER + SPACING;
                }
            }
            outstr.push_str(TERMINATING_CONNECTOR);
            match toml_val {
                TOMLType::Array(a) => outstr.push_str("Array"),
                TOMLType::AoT(_) => outstr.push_str("Array of Tables"),
                TOMLType::InlineTable(_) => outstr.push_str("Inline Table"),
                TOMLType::MultiStr(_) => outstr.push_str("Multi-line String"),
                TOMLType::MultiLitStr(_) => outstr.push_str("Multi-line Literal String"),
                TOMLType::LitStr(s) => outstr.push_str(s),
                TOMLType::BasicStr(s) => outstr.push_str(s),
                TOMLType::Bool(bl) => outstr.push_str(bl.to_string().as_str()),
                TOMLType::Int(i) => outstr.push_str(i.to_string().as_str()),
                TOMLType::Float(f) => outstr.push_str(f.to_string().as_str()),
                TOMLType::TimeStamp(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::NaiveDateTime(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::Date(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::Time(dt) => outstr.push_str(dt.to_string().as_str()),
                TOMLType::HTable(_) | TOMLType::DKTable(_) => (),
            }
        }
        outstr
    }
}
impl std::fmt::Display for ParsedTOML {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree())
    }
}

