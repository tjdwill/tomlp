// Module Declarations
mod constants; // Characters of Interest
mod parsedtoml; // The completely-parsed TOML table.
mod parsetools; // Tools that make the parsing operation easier for me to think about
mod tomlparse; // The TOML parser
mod tomltypes; // Rust representations of TOML types

// Imports
pub use parsedtoml::ParsedTOML;
use tomlparse::TOMLParser;
pub use tomltypes::{TOMLTable, TOMLType, ValFromTOMLKey};

/// The interface to the TOML parser.
/// Takes a string slice representing either an absolute path or a path relative to the current working directory.
/// The file must have extension `.toml`.
///
/// Ex. Given some TOML:
///
/// ```toml
/// # test.toml
///            # this is a valid comment.
/// [package]
/// name = "tomlp"
/// version = "0.1.0"
/// edition = "2021"
///
/// [dependencies]
/// unicode-segmentation = "~1.11.0"
/// chrono = "0.4.38"
///
/// [lib]
/// name = "tomlp"
/// path = "src/lib.rs"
///
/// [[bin]]
/// name = "prototype"
///     path = "src/bin.rs"
///
///
/// [[bin]]
/// name = 123
///     path = "src/bin.rs"
///
/// ```
///
/// Parse via
///
/// ```
/// use tomlp::parse;
/// let result = parse("test.toml")?;
/// println!("{}", result);
/// Ok(())
/// ```
///
/// This results in:
///
/// ```console
/// /
/// ├── package
/// │   ├── version
/// │   │   └── 0.1.0
/// │   ├── edition
/// │   │   └── 2021
/// │   └── name
/// │       └── tomlp
/// ├── lib
/// │   ├── name
/// │   │   └── tomlp
/// │   └── path
/// │       └── src/lib.rs
/// ├── bin (Arr_of_Tbls)
/// │   ├── path
/// │   │   └── src/bin.rs
/// │   └── name
/// │       └── prototype
/// │   ├── name
/// │   │   └── 123
/// │   └── path
/// │       └── src/bin.rs
/// └── dependencies
///     ├── unicode-segmentation
///     │   └── ~1.11.0
///     └── chrono
///         └── 0.4.38
/// ```
pub fn parse(file: &str) -> Result<ParsedTOML, String> {
    let mut parser = TOMLParser::init(file)?;
    let table = parser.parse_toml()?;
    Ok(ParsedTOML::from(table))
}
