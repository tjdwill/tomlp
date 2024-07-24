/// Prints all invalid chars for TOML strings.
/// Needed for instantiating a const array.
fn main() {
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
