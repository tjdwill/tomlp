/// Testing out my idea for parsing all of the local datetime formats in TOML
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime}; // 0.4.38

fn main() {
    test_date_parse();
}

fn try_naive_dtparse(s: &str) -> Option<Token> {
    if let Ok(val) = DateTime::parse_from_rfc3339(s) {
        Some(Token::DateTime(val))
    } else if let Some(val) = try_naive_datetime(s) {
        Some(Token::NDateTime(val))
    } else if let Some(val) = try_naive_date(s) {
        Some(Token::NDate(val))
    } else if let Some(val) = try_naive_time(s) {
        Some(Token::NTime(val))
    } else {
        None
    }
}

fn try_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    const NAIVEDATETIME_FORMATS: [&str; 4] = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S.%f",
        "%Y-%m-%dT%H:%M:%S.%f",
    ];

    for format in NAIVEDATETIME_FORMATS {
        match NaiveDateTime::parse_from_str(s, format) {
            Ok(val) => return Some(val),
            Err(_) => (),
        }
    }
    None
}

fn try_naive_date(s: &str) -> Option<NaiveDate> {
    const NAIVEDATE_FORMAT: &str = "%Y-%m-%d";

    match NaiveDate::parse_from_str(s, NAIVEDATE_FORMAT) {
        Ok(val) => return Some(val),
        Err(_) => None,
    }
}

fn try_naive_time(s: &str) -> Option<NaiveTime> {
    const NAIVETIME_FORMATS: [&str; 2] = ["%H:%M:%S.%f", "%H:%M:%S"];

    for format in NAIVETIME_FORMATS {
        match NaiveTime::parse_from_str(s, format) {
            Ok(val) => return Some(val),
            Err(_) => (),
        }
    }
    None
}

fn test_date_parse() {
    let test_bank: [&str; 10] = [
        "1979-05-27T00:32:00.999999-07:00",
        "1979-05-27T07:32:00Z",
        "1979-05-27T00:32:00-07:00",
        "1979-05-27T00:32:00.999999",
        "1979-05-27T00:32:00",
        "1979-05-27 00:32:00.999999",
        "1979-05-27 00:32:01",
        "1979-05-27",
        "07:32:00",
        "00:32:00.999999",
    ];
    for datetime in test_bank {
        let parsed = try_naive_dtparse(datetime);
        match parsed {
            Some(dt) => println!("Parsed Output: {:?}", dt),
            None => println!("No DateTime found."),
        }
    }
}

#[derive(Debug)]
enum Token {
    DateTime(DateTime<FixedOffset>),
    NDateTime(NaiveDateTime),
    NDate(NaiveDate),
    NTime(NaiveTime),
}
