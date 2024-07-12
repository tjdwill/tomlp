use tomlp::prototype::parserline::{ParserLine, TOMLSegments};

fn main() {
    print_line_segments("some_key = value # This is a comment!\n");
    print_line_segments("str_key = \"A String\" # This is a comment!\n");
    print_line_segments(
        "str_key = \"\"\"\nA multi-String\"\"\" # This is a comment!\n"
    );
    print_line_segments("\n");
    print_line_segments("# A Comment   \n");
    print_line_segments("[A Table]\n");
    print_line_segments("[[Array of Tables]]\n");
    print_line_segments("{Inline}\n");
    
}

fn print_line_segments(input: &str) {
    let mut pl = ParserLine::from(input);
    pl.find_segments();
    println!("Input: {}", input);
    for segment in &pl {
        println!("{:?}", segment.collect::<Vec<_>>());
    }
    println!("\n");
}