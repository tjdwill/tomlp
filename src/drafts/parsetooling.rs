#![allow(dead_code, unused_imports, unused_variables)]
fn main() {
    tests::main();
}

pub mod tomlparse {
    use super::*;
    static EOF_ERROR: &str = "End of File during parsing operation.";

    #[derive(Debug)]
    pub struct TOMLParser {
        buffer: String,
        reader: BufReader<File>,
        line_num: usize,
    }
    impl TOMLParser {
        ////////////////////////
        // Creation/Modification
        ////////////////////////
        pub fn init(file_path: &str) -> Result<Self, String> {
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
                Some(ext) => {
                    if !test.exists() {
                        return Err("File does not exist.".to_string());
                    } else if ext != toml_ext {
                        return Err("Incorrect file extension.".to_string());
                    } else {
                    }
                }
                None => return Err("Incorrect file extension.".to_string()),
            }

            match File::open(input) {
                Ok(fd) => Ok(fd),
                Err(err) => Err(format!("File Open Error: {}", err.kind())),
            }
        }

        /// returns false -> EoF
        /// Won't check for EoF mid-value parsing.
        /// I only plan to check in the outer loop.
        pub fn next_line(&mut self) -> Result<bool, String> {
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

        fn gen_pline(&self) -> ParserLine {
            ParserLine::new(self.buffer.clone(), self.line_num)
        }

        ////////////////////
        // Parsing Functions
        ////////////////////

        /// Produce a UTF8 escape literal from a given iterator
        /// Assumes Unix OS so the newline can fit into a char.
        /// In the future (when I get this thing working), I can adjust for platform support
        /// which would likely require this signature to change to return &'static str OR pass
        /// the String structure in directly to push the slice to it.
        /// This function is for DRAFTING purposes ONLY. The signature will be different in the implementation.
        pub fn process_multi_escape_sequence(
            &mut self,
            mut pline: ParserLine,
        ) -> Result<(char, ParserLine), String> {
            // TODO: Check logic for this function
            // Assume we have identified a backslash
            let mut seg = {
                match pline.next_seg() {
                    Some(next_seg) => next_seg,
                    None => return Err(String::from(EOF_ERROR)), // only condition in which a \ is followed by nothing.
                }
            };
            // from here, we *know* there is at least one character left to read.
            let outchar: char;
            let c = seg.next().unwrap();
            match c {
                "b" => outchar = '\u{0008}',
                "t" => outchar = '\t',
                "n" => outchar = '\n',
                "f" => outchar = '\u{000C}',
                "r" => outchar = '\r',
                "\"" => outchar = '\"',
                "\\" => outchar = '\\',
                "u" | "U" => {
                    match Self::escape_utf8(&mut seg) {
                        Some(c) => outchar = c,
                        None => return Err(format!(
                            "Err: Line {}: Invalid UTF8 escape sequence. Format: \\uXXXX or \\uXXXXXXXX", pline.line_num
                        ))
                    }
                }
                _ => {
                    if !c.chars().next().unwrap().is_whitespace() {
                        return Err(format!(
                            "Error: Line {}: Invalid escape sequence.", pline.line_num
                        ))
                    } else {
                        // find next non-whitespace char
                        let count = seg.count();
                        return self.get_nonwhitespace(ParserLine::continuation(pline, count))
                    }
                }
            }
            let count = seg.count();
            return Ok((outchar, ParserLine::continuation(pline, count)));
        }

        pub fn get_nonwhitespace(
            &mut self,
            mut pline: ParserLine,
        ) -> Result<(char, ParserLine), String> {
            // The last line may have ended if the whitespace character was a newline, so
            // the next line is obtained in that instance.
            let mut seg = {
                match pline.next_seg() {
                    Some(next_seg) => next_seg,
                    None => {
                        if !self.next_line()? {
                            return Err(String::from(EOF_ERROR));
                        } else {
                            return self.get_nonwhitespace(self.gen_pline());
                        }
                    }
                }
            };
            // find the non-newline
            loop {
                match seg.next() {
                    Some(ch) => {
                        let ch = ch.chars().next().unwrap();
                        if !ch.is_whitespace() {
                            let count = seg.count();
                            return Ok((ch, ParserLine::continuation(pline, count)));
                        } else {
                            continue;
                        }
                    }
                    None => {
                        // try to get the next segment
                        match pline.next_seg() {
                            Some(next_seg) => {
                                seg = next_seg;
                                continue;
                            }
                            None => {
                                // try to get the next line
                                if !self.next_line()? {
                                    return Err(String::from(EOF_ERROR));
                                } else {
                                    return self.get_nonwhitespace(self.gen_pline());
                                }
                            }
                        }
                    }
                }
            }
        }

        pub fn escape_utf8(iter: &mut TOMLSeg<'_>) -> Option<char> {
            // try to find 4 or 8 hexadecimal digits
            const MIN_SEQ_LENGTH: i32 = 4;
            const MAX_SEQ_LENGTH: i32 = 8;

            let mut hex_val = 0_u32;
            let mut digits_processed = 0;

            while digits_processed < MAX_SEQ_LENGTH {
                if Self::is_hexdigit(iter.peek()) {
                    let digit = iter.next().unwrap();
                    hex_val = 16 * hex_val + u32::from_str_radix(digit, 16).unwrap();
                } else if digits_processed == MIN_SEQ_LENGTH {
                    break;
                } else {
                    return None;
                }
                digits_processed += 1;
            }

            std::char::from_u32(hex_val)
        }

        /// determines if the next entry of a UTF8Peek Iterator is a hexadecimal value
        fn is_hexdigit(query: Option<&&str>) -> bool {
            match query {
                Some(text) => {
                    let char_opt = text.chars().next();
                    match char_opt {
                        Some(c) => match c {
                            '0'..='9' | 'A'..='F' | 'a'..='f' => true,
                            _ => false,
                        },
                        None => false, // was the empty string
                    }
                }
                None => false,
            }
        }
    } // impl ParserLine

    ///////////////////
    // Helper Functions
    ///////////////////

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

    pub fn process_comment(mut pline: ParserLine) -> Result<(), String> {
        let line_num = pline.line_num;
        let mut iter = pline.next_seg().unwrap();
        loop {
            match iter.next() {
                Some(ch) => {
                    if !is_valid_comment(ch) {
                        return Err(format!(
                            "Invalid Comment Character: {} on Line {}",
                            ch, line_num
                        ));
                    }
                }
                None => {
                    let next_iter = {
                        match pline.peek() {
                            Some(iter) => iter,
                            None => return Ok(()),
                        }
                    };
                    return process_comment(pline);
                }
            }
        }
    }

    pub fn is_valid_comment(c: &str) -> bool {
        true
    }
}

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
    pub line_num: usize,
    data: String,
    // iteration things
    seg_nums: Vec<usize>, // a vector of what is essentially cursor positions to denote segment ranges.
    iter_limit: usize,    // The iteration termination value
    curr_seg_num: usize,  // x: 0 <= x <= iter_limit;
    remaining_graphemes: usize, // a tracker for reproducing a given segment with some offset.
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
            remaining_graphemes: 0,
        }
    }

    pub fn continuation(pline: Self, count: usize) -> Self {
        Self {
            remaining_graphemes: count,
            ..pline
        }
    }

    pub fn iter<'a>(&'a self) -> UTF8Peekable<'a> {
        self.data.as_str().graphemes(true).peekable()
    }

    /// Returns the current item without advancing the incrementer.
    pub fn peek(&self) -> Option<TOMLSeg<'_>> {
        let remaining_graphs = self.remaining_graphemes;
        let cursors = &self.seg_nums;

        // Produce a continuation iterator for a segment offset.
        /*
            REASONING CHECK: since a continuation is only called when an iterator
            has already been produced at least once, in order to produce an offset iterator,
            we must decrement the current segment number to produce it.

            The ONLY way I can see this producing an error is if offset iterator is produced when
            curr_seg_num is 0. Assuming I only produce the remaining grapheme count with the current
            segment iterator, this would only occur if said iterator were produced with peek.

            In other words, only produce an offset iterator with iterators produced from next_seg.
        */
        if remaining_graphs != 0 {
            let curr_num = self.curr_seg_num - 1;
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            let num_elements = ub - lb;
            let skips = num_elements - remaining_graphs;

            let mut iter = self
                .data
                .as_str()
                .graphemes(true)
                .take(ub)
                .skip(lb)
                .peekable();
            for _ in 0..skips {
                iter.next();
            }
            return Some(iter);
        }

        let output: Option<TOMLSeg<'_>>;
        let curr_num = self.curr_seg_num;
        if curr_num == self.iter_limit {
            output = None;
        } else {
            // produce full segment
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            output = Some(
                self.data
                    .as_str()
                    .graphemes(true)
                    .take(ub)
                    .skip(lb)
                    .peekable(),
            );
        }

        output
    }

    pub fn next_seg(&mut self) -> Option<TOMLSeg<'_>> {
        let remaining_graphs = self.remaining_graphemes;
        let cursors = &self.seg_nums;

        // Produce a continuation iterator for a segment offset.
        /*
            REASONING CHECK: since a continuation is only called when an iterator
            has already been produced at least once, in order to produce an offset iterator,
            we must decrement the current segment number to produce it.

            The ONLY way I can see this producing an error is if offset iterator is produced when
            curr_seg_num is 0. Assuming I only produce the remaining grapheme count with the current
            segment iterator, this would only occur if said iterator were produced with peek.

            In other words, only produce an offset iterator with iterators produced from next_seg.
        */
        if remaining_graphs != 0 {
            let curr_num = self.curr_seg_num - 1;
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            let num_elements = ub - lb;
            let skips = num_elements - remaining_graphs;

            let mut iter = self
                .data
                .as_str()
                .graphemes(true)
                .take(ub)
                .skip(lb)
                .peekable();
            for _ in 0..skips {
                iter.next();
            }
            self.remaining_graphemes = 0; // set termination cond.
            return Some(iter);
        }

        let output: Option<TOMLSeg<'_>>;
        let curr_num = self.curr_seg_num;
        if curr_num == self.iter_limit {
            output = None;
        } else {
            // produce full segment
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            output = Some(
                self.data
                    .as_str()
                    .graphemes(true)
                    .take(ub)
                    .skip(lb)
                    .peekable(),
            );
            self.curr_seg_num += 1;
        }

        output
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
            remaining_graphemes: 0,
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

mod tests {
    use super::ParserLine;
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
