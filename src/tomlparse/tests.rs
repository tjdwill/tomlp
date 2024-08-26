#![cfg(test)]
use std::path::Path;

use super::{KeyVal, ParserLine, TOMLParser, TOMLTable, TOMLType, TPath};
type TestReturn = Result<(), String>;

// TESTS
#[test]
fn test_parser() -> TestReturn {
    const FILES: [&str; 4] = [
        "test_resources/blank.toml",
        "test_resources/test.toml",
        "test_resources/spec-example-1.toml",
        "test_resources/ripgrep.toml",
    ];
    //const FILES: [&str; 1] = [ "test_resources/blank.toml" ];

    for file_str in FILES {
        let source_dir = match Path::new(file!()).canonicalize() {
            Ok(s) => s,
            _ => return Err(String::from("File error.")),
        };
        let file = source_dir.parent().unwrap().join(file_str);
        println!("File: {}", file.to_str().unwrap());
        let mut parser = TOMLParser::init(file.to_str().unwrap())?;
        let table = parser.parse_toml()?;
        println!("Parsed TOML:\n{:?}\n", table);
    }
    Ok(())
}

#[test]
fn parse_tableheads() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/table_heads.toml");

    let mut parser = TOMLParser::init(file.to_str().unwrap())?;

    let mut table = TOMLTable::new();
    while let Ok(pline) = parser.next_parserline() {
        parser.parse_table_header(pline, &mut table)?;
        // println!("Table:\n{:?}", parser.view_table());
    }
    println!("Table:\n{:?}", &table);
    Ok(())
}

#[test]
fn test_insertion() -> TestReturn {
    let mut table = TOMLTable::new();
    let table_head = &mut table;
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/keyvals.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    while let Ok(pline) = parser.next_parserline() {
        let (key_val, _) = parser.parse_keyval(pline)?;
        TOMLParser::insert(key_val, table_head)?;
    }
    println!("Parsed Table: {:?}", table);
    Ok(())
}

#[test]
fn key_vals() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/keyvals.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    let mut successes = 0;
    while let Ok(pline) = parser.next_parserline() {
        let (KeyVal(key, val), _) = parser.parse_keyval(pline)?;
        println!("Key: {key:?}\nVal: {val:?}\n");
        successes += 1;
    }
    assert!(successes != 0);
    Ok(())
}

#[test]
fn parse_keys() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/keys.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    while let Ok(pline) = parser.next_parserline() {
        let (path, _) = parser.parse_key(pline)?;
        println!("Parsed Key: {:?}", path);
    }
    Ok(())
}

#[test]
fn arrays() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/arrays.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    while let Ok(pline) = parser.next_parserline() {
        let (array, _) = parser.parse_array(pline)?;
        println!("Array: {:?}", array);
    }
    Ok(())
}

#[test]
fn inline_tables() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/inline_tables.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    while let Ok(pline) = parser.next_parserline() {
        let (table, _pline) = parser.parse_inline_table(pline)?;
        println!("Parsed Inline Table: {:?}", table);
    }
    Ok(())
}

#[test]
fn atomic_values() -> TestReturn {
    const NUM_VALUES: i32 = 33; // total number of values in test file.
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/values.toml");

    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    let mut successes = 0;
    while let Ok(pline) = parser.next_parserline() {
        parser.parse_value(pline)?;
        successes += 1;
    }
    assert_eq!(successes, NUM_VALUES);
    Ok(())
}

#[test]
fn bools() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/bool.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    let mut test = false;
    // Invariant: `test == !parsed_value`
    while let Ok(pline) = parser.next_parserline() {
        let (boolean, _pline) = TOMLParser::parse_bool(pline)?;
        if let TOMLType::Bool(val) = boolean {
            assert_eq!(val, !test);
            test = val;
        }
    }
    Ok(())
}

#[test]
fn numerics() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/numerics.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;

    while let Ok(pline) = parser.next_parserline() {
        TOMLParser::parse_numeric(pline)?;
    }
    Ok(())
}

#[test]
fn datetime() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/dates.toml");

    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    println!("\nDate Parsing:");
    while let Ok(pline) = parser.next_parserline() {
        let (date, _pline) = TOMLParser::parse_date(pline)?;
        println!("{:?}", date);
    }
    Ok(())
}

#[test]
fn floats() -> TestReturn {
    test_float()?;
    nan_test()?;
    invalid_format()
}
fn test_float() -> TestReturn {
    const FLOAT_STRS: [&str; 13] = [
        "224_627.445_991_228",
        "-0.0",
        "+0.0",
        "inf",
        "+inf",
        "-inf",
        "+1.0",
        "3.1415",
        "-0.01",
        "5e+22",
        "1e06",
        "-2E-2",
        "6.626e-34",
    ];
    const FLOATS: [f64; 13] = [
        224627.445991228,
        -0.0,
        0.0,
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1.0,
        3.1415,
        -0.01,
        5e22,
        1e6,
        -2e-2,
        6.626e-34,
    ];

    let mut i = 0;
    for s in FLOAT_STRS {
        let pline = ParserLine::new(s.to_string(), i);
        let result = TOMLParser::parse_float(pline)?.0;
        match result {
            TOMLType::Float(val) => assert_eq!(FLOATS[i], val),
            _ => return Err(String::from("Will never reach here.")),
        }
        i += 1;
    }
    Ok(())
}
fn nan_test() -> TestReturn {
    const NAN_STRS: [&str; 3] = ["nan", "+nan", "-nan"];
    for s in NAN_STRS {
        let pline = ParserLine::new(s.to_string(), 0);
        let result = TOMLParser::parse_float(pline)?.0;
        match result {
            TOMLType::Float(val) => assert_eq!(true, val.is_nan()),
            _ => return Err(String::from("Will never reach here.")),
        }
    }
    Ok(())
}
fn invalid_format() -> TestReturn {
    const BAD_F64: [&str; 3] = [".12", "3.e+20", "12."];
    for s in BAD_F64 {
        let pline = ParserLine::new(s.to_string(), 0);
        let result = TOMLParser::parse_float(pline);
        match result {
            Err(_msg) => {
                // println!("{_msg}");
            }
            _ => return Err(String::from("Failed to catch invalid format.")),
        }
    }

    Ok(())
}

#[test]
fn ints() -> TestReturn {
    hundred_twenty_three()?;
    negatives()
}
fn hundred_twenty_three() -> TestReturn {
    const HUNDRED_TWENTY_THREES: [&str; 9] = [
        " 123\n",
        "1_2_3",
        "12_3",
        "0x7b",
        "\t0x0007B",
        "0x07_b",
        "0o173",
        "0b01111011",
        "0b111_1011",
    ];
    for form in &HUNDRED_TWENTY_THREES {
        let pline = ParserLine::new(form.to_string(), 0);
        let parsed_val = TOMLParser::parse_integer(pline)?.0;
        match parsed_val {
            TOMLType::Int(val) => assert_eq!(123, val),
            _ => return Err("Should never happen.".to_string()),
        }
    }
    Ok(())
}
fn negatives() -> TestReturn {
    const SIZE: usize = 5;
    const NEGATIVES: [&str; SIZE] = ["-0", "-123", "-27", "-3567", "-562"];
    const NEGATIVE_INTS: [i64; SIZE] = [0, -123, -27, -3567, -562];

    let mut i = 0;
    for s in &NEGATIVES {
        let pline = ParserLine::new(s.to_string(), i);
        let parsed_val = TOMLParser::parse_integer(pline)?.0;
        match parsed_val {
            TOMLType::Int(val) => assert_eq!(NEGATIVE_INTS[i], val),
            _ => return Err("Should never happen.".to_string()),
        }
        i += 1;
    }
    Ok(())
}

#[test]
fn strings() -> TestReturn {
    use super::{LITERAL_STR_TOKEN, STR_TOKEN};
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/strings.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;
    let mut pline = parser.next_parserline()?;

    // Basic Strings
    println!("\nBasic Strings");
    while let Some(&STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("\nLine {}: ", pline.line_num());
        let (outstring, context) = parser.parse_string(pline)?;
        if let TOMLType::BasicStr(str) = outstring {
            let outstring = str;
            println!("Basic String\n{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()));
        }
        pline = parser.next_parserline()?;
    }

    // Multi-Strings
    println!("\nMulti-line Strings");
    pline = parser.next_parserline()?;
    while let Some(&STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.parse_string(pline)?;
        if let TOMLType::MultiStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()));
        }
        pline = parser.next_parserline()?;
    }

    // Literal Strings
    println!("\nLiteral Strings");
    pline = parser.next_parserline()?;
    while let Some(&LITERAL_STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.parse_literal_string(pline)?;
        if let TOMLType::LitStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()));
        }
        pline = parser.next_parserline()?;
    }

    // Multi-Literal Strings
    println!("\nMulti-line Literal Strings");
    pline = parser.next_parserline()?;
    while let Some(&LITERAL_STR_TOKEN) = pline.peek().unwrap().peek() {
        print!("Line {}: ", pline.line_num());
        let (outstring, context) = parser.parse_literal_string(pline)?;
        if let TOMLType::MultiLitStr(str) = outstring {
            let outstring = str;
            println!("{}", outstring);
        } else {
            return Err(format!("Line {}", context.line_num()));
        }
        pline = parser.next_parserline()?;
    }

    Ok(())
}

#[test]
fn escape_sequences() -> TestReturn {
    let source_dir = match Path::new(file!()).canonicalize() {
        Ok(s) => s,
        _ => return Err(String::from("File error.")),
    };
    let file = source_dir
        .parent()
        .unwrap()
        .join("test_resources/blank.toml");
    let mut parser = TOMLParser::init(file.to_str().unwrap())?;

    let eof = "\n".to_string();
    let fire = "u0001f525".to_string();
    let next_nonws = "    \n\n\nt".to_string();

    assert_eq!(
        '🔥',
        parser
            .parse_multi_escape_sequence(ParserLine::new(fire, 0))?
            .0
    );
    assert_eq!(
        't',
        parser
            .parse_multi_escape_sequence(ParserLine::new(next_nonws, 0))?
            .0
    );
    assert_eq!(
        true,
        parser
            .parse_multi_escape_sequence(ParserLine::new(eof, 0))
            .is_err()
    );
    Ok(())
}

/// Prints all invalid chars for TOML strings.
/// Needed for instantiating a const array.
/// Placing it here as I don't know where else to put it.
/// Uncomment the test directive and run it with `--show-output`.
//#[test]
fn invalid_strs() {
    print_invalid_str_graphemes()
}

fn get_graphemes(s: &str) -> Vec<&str> {
    unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect::<Vec<_>>()
}
/// Use this to print an array of invalid graphemes. Make the resulting array via Copy/Paste.
pub fn print_invalid_str_graphemes() {
    println!("Invalid TOML str grapheme Report");
    let inval_string = get_invalid_str_graphemes();
    let invalids = get_graphemes(inval_string.as_str());
    println!("Total Num of Invalids: {}", invalids.len());
    println!("Invalid str graphemes:\n{:?}\n", invalids);
}

fn get_invalid_str_graphemes() -> String {
    let range1 = 0_u8..=8_u8;
    let range2 = u8::from_str_radix("A", 16).unwrap()..=u8::from_str_radix("1F", 16).unwrap();
    let range3 = u8::from_str_radix("7F", 16).unwrap()..u8::from_str_radix("80", 16).unwrap();
    let graphemes = range1.chain(range2.chain(range3)).collect::<Vec<u8>>();
    String::from_utf8(graphemes).unwrap()
}
