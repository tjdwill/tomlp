use tomlp::drafts::{parsetools::ParserLine, tokens::TOMLType, tomlparse::TOMLParser};

type TestRet = Result<(), String>;
fn main() -> TestRet {
    test_float()
}

fn test_float() -> TestRet {
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
    // f64::NAN, f64::NAN, f64::NAN,
    //  "nan", "+nan", "-nan",
}
