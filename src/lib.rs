// Module Declarations
mod constants; // Characters of Interest
mod parsedtoml; // The completely-parsed TOML table.
mod parsetools; // Tools that make the parsing operation easier for me to think about
mod tomlparse;
mod tomltypes; // Rust representations of TOML types // The TOML parser

// Imports
use parsedtoml::ParsedTOML;
use tomlparse::TOMLParser;
pub use tomltypes::{TOMLTable, TOMLType};

/// The interface to the TOML parser. 
/// Takes a string slice representing either an absolute path or a path relative to the current working directory.
/// The file must have extension `.toml`.
pub fn parse(file: &str) -> Result<ParsedTOML, String> {
    let mut parser = TOMLParser::init(file)?;
    let table = parser.parse_toml()?;
    Ok(ParsedTOML::from(table))
}
