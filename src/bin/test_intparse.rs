use tomlp::drafts::{
    parsetooling::ParserLine,
    tokens::TOMLType,
    tomlparse::TOMLParser,
};

type TestRet = Result<(), String>;
fn main() -> TestRet {
    hundred_twenty_three()?;
    negatives()    
}

fn hundred_twenty_three () -> TestRet {
    const HUNDRED_TWENTY_THREES: [&str; 9] = [
        " 123\n", "1_2_3", "12_3", 
        "0x7b", "\t0x0007B", "0x07_b", 
        "0o173",
        "0b01111011","0b111_1011",  
    ];  
    for form in &HUNDRED_TWENTY_THREES {
        let pline = ParserLine::new(form.to_string(), 0);
        let parsed_val = TOMLParser::parse_integer(pline)?.0;
        match parsed_val {
            TOMLType::Int(val) => assert_eq!(123, val),
            _ => return Err("Should never happen.".to_string())
        }
    }
    Ok(())
}

fn negatives () -> TestRet {
    const SIZE: usize = 5;
    const NEGATIVES: [&str; SIZE] = [
        "-0", "-123", "-27", "-3567", "-562"
    ];
    const NEGATIVE_INTS: [i64; SIZE] = [
       0, -123, -27, -3567, -562 
    ];

    let mut i = 0;
    for s in &NEGATIVES {
        let pline = ParserLine::new(s.to_string(), i);
        let parsed_val = TOMLParser::parse_integer(pline)?.0;
        match parsed_val {
            TOMLType::Int(val) => assert_eq!(NEGATIVE_INTS[i], val),
            _ => return Err("Should never happen.".to_string())
        }
        i += 1;
    }
    Ok(())
}