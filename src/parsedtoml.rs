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
    fn tree(view: &TOMLTable, level: usize,) -> String {
        todo!()
    }
}
