#![allow(dead_code, unused_imports, unused_variables)]
fn main() {
    parse_tooling::main();
}

mod tomlparse {
    use crate::parse_tooling::*;

    #[derive(Debug)]
    struct TOMLParser {
        buffer: String,
        reader: BufReader<File>,
        line_num: usize,
    }
    impl TOMLParser {
        ////////////////////////
        // Creation/Modification
        ////////////////////////
        fn init(file_path: &str) -> Result<Self, String> {
            let fd = Self::validate_file(file_path)?;
            Ok(Self {
                buffer: String::with_capacity(100 * 4),
                reader: BufReader::new(fd),
                line_num: 0,
            })
        }

        fn validate_file(input: &str) -> Result<File, String> {
            use std::ffi::OsStr;

            let toml_ext: &OsStr = OsStr::new("toml");
            let test = Path::new(input);
            match test.extension() {
                Some(toml_ext) => {
                    if !test.exists() {
                        return Err("File does not exist.".to_string());
                    } else {
                    }
                }
                _ => return Err("Incorrect file extension.".to_string()),
            }

            match File::open(input) {
                Ok(fd) => Ok(fd),
                Err(err) => Err(format!("File Open Error: {}", err.kind())),
            }
        }

        /// returns false -> EoF
        /// Won't check for EoF mid-value parsing.
        /// I only plan to check in the outer loop.
        fn next_line(&mut self) -> Result<bool, String> {
            self.buffer.clear();
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => return Ok(false),
                Ok(sz) => {
                    self.line_num += 1;
                    return Ok(true);
                }
                Err(err) => {
                    return Err(format!(
                        "Read error for line {1}: {0}",
                        err.kind(),
                        self.line_num + 1
                    ))
                }
            }
        }

        ////////////////////
        // Parsing Functions
        ////////////////////
        /*
            There are three levels to the iteration structures:
                2. TOMLSeg<'a>: the current segment of the current line
                1. ParserLine: The structure that produces `TOMLSeg`s
                0. Self: The structure that produces a ParserLine

            One current concern is how to properly update these structures
            as a given semantic item is parsed.

            Let's say I am in the middle of
            processing a comment and run out of elements in the TOMLSeg (called var).
            I need to ensure I am truly at the end of the line. To do so:
                - Grab the next element of ParserLine.
                - If this item is Some(iter), continue processing by making
                var: TOMLSeg<'a> = iter.
                - If in the None case, we're done processing the comment.

            Simple enough, the only two structures needed in this case are
            `TOMLSeg` and `ParserLine`. This, however, is because a comment
            context is only valid for that given line. If we are processing a
            comment, the current ParserLine need only be considered.

            If, however, a multi-line structure were considered (ex. a multi-string or an array),
            the process is more involved. In this case, the line itself may need to be updated:

            ```
            <'a> {
                let mut tomlseg: TOMLSeg<'a>;
                let mut pline: ParserLine:
                let mut parser: TOMLParser;

                toml.seg.next() -> None; Ok, try to get the next TOMLSeg:
                tomlseg = pliter.next();    // Say this is None, then we need the next line.
                parser.next_line()?;  // NOTE: check that we are not EoF as well.
                pline = parser.context.into_iter();
                ... keep processing.
            }
            ```
            As one can see, in a multi-line context, all entites are able to be
            modified. The parser may need to be incremented to the next line.
            which is then used to create a new ParserLine which then updates the TOMLSeg.
        */
        fn process_comment<'a>(iter: TOMLSeg<'a>) {}

        fn compile_test(&mut self, mut context: ParserLine,) -> Result<ParserLine, String> {
            let mut iter = context.next_seg().unwrap();
            loop {
                match iter.next() {
                    // Check the current segment
                    Some(ch) => {
                        println!("Char: {}", ch);
                        if ch == ";" {
                            return Ok(context);
                        }
                    }
                    None => {
                        // try to get the next segment
                        match context.next_seg() {
                            Some(seg) => {
                                iter = seg;
                                continue;
                            }
                            None => {
                                // try to get the next line
                                let read_bytes: bool = self.next_line()?;
                                if !read_bytes {
                                    return Err(String::from("Reached End of File while parsing."));
                                } else {
                                    let pline = self.gen_context();
                                    return self.compile_test(pline);
                                }
                            }
                        }
                    }
                }
            }
        }

        fn gen_context(&self) -> ParserLine {
            ParserLine::new(self.buffer.clone(), self.line_num)
        }
    }
}

pub mod parse_tooling {

    pub use std::fs::File;
    pub use std::io::{prelude::*, BufReader};
    use std::iter::{Peekable, Skip, Take};
    pub use std::path::Path;
    use unicode_segmentation::{Graphemes, UnicodeSegmentation as utf8};

    pub type UTF8Peekable<'a> = Peekable<Graphemes<'a>>;
    pub type TOMLSeg<'a> = Peekable<Skip<Take<Graphemes<'a>>>>;

    //////////////
    // Struct Defs
    //////////////

    #[derive(Debug)]
    pub struct ParserLine {
        data: String,
        line_num: usize,
        // iteration things
        seg_nums: Vec<usize>,
        iter_limit: usize,
        curr_seg_num: usize,
    }
    impl ParserLine {
        pub fn new(input: String, line_num: usize) -> Self {
            let seg_nums = Self::find_segments(input.as_str());
            // the iter_limit is set to 1 less than the number of elements in
            // the seg_num vector. This is because in the actual iteration, I
            // poll seg_num[i] and seg_num[i+1] in a given iteration.
            let iter_limit = std::cmp::max(seg_nums.len() - 1, 0);
            Self {
                data: input,
                line_num,
                seg_nums,
                iter_limit,
                curr_seg_num: 0,
            }
        }

        pub fn iter<'a>(&'a self) -> UTF8Peekable<'a> {
            self.data.as_str().graphemes(true).peekable()
        }
        
        /// Returns the current item without advancing the incrementer.
        pub fn peek(&self) -> Option<TOMLSeg<'_>> {
            let curr_num = self.curr_seg_num;
            if curr_num == self.iter_limit {
                return None;
            }

            let cursors = &self.seg_nums;
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            // assert!(lb < ub);  // removed for performance #premature_opt.

            Some(
                self.data
                    .as_str()
                    .graphemes(true)
                    .take(ub)
                    .skip(lb)
                    .peekable()
            )
        }
        
        pub fn next_seg(&mut self) -> Option<TOMLSeg<'_>> {
            let curr_num = self.curr_seg_num;
            if curr_num == self.iter_limit {
                return None;
            }

            let cursors = &self.seg_nums;
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            self.curr_seg_num += 1;
            // assert!(lb < ub);  // removed for performance #premature_opt.

            Some(
                self.data
                    .as_str()
                    .graphemes(true)
                    .take(ub)
                    .skip(lb)
                    .peekable()
            )
        }
        
        /////////////////
        // Static Methods
        /////////////////

        pub fn replacement() -> Self {
            Self {
                data: String::new(),
                seg_nums: Vec::new(),
                line_num: 0,
                iter_limit: 0,
                curr_seg_num: 0,
            }
        }
        
        /// segment the current line semantically
        /// Given some
        ///     key = value # comment
        /// this function should find segments that represent the following split:
        ///     |key |=| value |# comment|
        fn find_segments(input_str: &str) -> Vec<usize> {
            let mut seg_spots: Vec<usize> = vec![];
            let mut graph_count = 0;
            let mut skip = false;
            for (i, graph) in input_str.graphemes(true).enumerate() {
                graph_count += 1;
                if skip {
                    skip = false;
                    continue;
                }
                match graph {
                    "[" | "]" | "#" | "{" | "}" | "," => seg_spots.push(i),
                    "=" => {
                        seg_spots.push(i);
                        seg_spots.push(i + 1);
                        skip = true;
                    }
                    _ => {
                        if i == 0 {
                            seg_spots.push(0);
                        }
                    }
                }
            }
            if !seg_spots.is_empty() {
                // Add endpoint
                seg_spots.push(graph_count);
            }
            seg_spots
        }
    }
    
    //////////
    // Helpers
    //////////

    pub fn skip_ws<'a>(iter: &mut TOMLSeg<'a>) {
        loop {
            match iter.peek() {
                Some(&ch) => match ch {
                    " " | "\t" => {
                        iter.next();
                        continue;
                    }
                    _ => return,
                },
                None => return,
            }
        }
    }

    /////////////
    // Functions
    /////////////
    pub fn main() {
        let s = "==#[]{}".to_string();

        print_iters(s.as_str());
        print_iters("some_key = value # this is a comment\n");
        print_iters("[\"This is a normal table key\"]\n");
        print_iters("{InlineTableKey: [An array, of, items]}\n");
    }

    fn print_iters(s: &str) {
        let mut iters = ParserLine::new(s.to_string(), 0);
        println!("Iters: {:?}", iters);
        while let Some(mut iter) = iters.next_seg() {
            println!("Next Iter: {:?}", iter);
            iter.peek();
            let vector = iter.collect::<Vec<_>>();
            println!("IterItem: {:?}", vector);
        }
        println!("\n");
    }
}
