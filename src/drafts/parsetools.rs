fn main() {
    tests::main();
}

//stdlib imports
use std::iter::Peekable;
// third-party imports
use unicode_segmentation::{Graphemes, GraphemeIndices, UnicodeSegmentation as utf8};
// internal imports
use super::constants::{
    COMMENT_TOKEN, INLINETAB_CLOSE_TOKEN, INLINETAB_OPEN_TOKEN, KEY_VAL_SEP, LITERAL_STR_TOKEN,
    SEQUENCE_DELIM, STR_TOKEN, TABLE_CLOSE_TOKEN, TABLE_OPEN_TOKEN,
};

//////////////
// Struct Defs
//////////////
#[derive(Debug)]
/// An interable line segment for TOML Parsing.
pub struct TOMLSeg<'a> {
    content: &'a str,
    iter: Peekable<Graphemes<'a>>,
}
impl<'a> TOMLSeg<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            content: s,
            iter: s.graphemes(true).peekable(),
        }
    }

    // retrieve a reference to the full segment str slice
    pub fn content(&self) -> &str {
        self.content
    }
    
    // Retrieve a preview of the next iterable item.
    pub fn peek(&mut self) -> Option<&&str> {
        self.iter.peek()
    }
}
impl<'a> Iterator for TOMLSeg<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> Default for TOMLSeg<'a> {
    fn default() -> Self {
        Self::new("")
    }
}

/// A type for maintaining the parser's state.
#[derive(Debug, Clone)]
pub struct ParserLine {
    data: String,               // the current line
    line_num: usize,            // line's file location
    // iteration things
    seg_nums: Vec<usize>,       // a vector of what is essentially cursor positions to denote segment ranges.
    byte_nums: Vec<usize>,      // a vector of byte offsets to enable segment slice construction
    iter_limit: usize,          // The iteration terminal value
    curr_seg_num: usize,        // x: 0 <= x <= iter_limit;
    remaining_graphemes: usize, // a tracker for reproducing a given segment with some offset.
}
impl ParserLine {
    pub fn new(input: String, line_num: usize) -> Self {
        let (seg_nums, byte_nums) = Self::find_segments(input.as_str());
        // the iter_limit is set to 1 less than the number of elements in
        // the seg_num vector. This is because in the actual iteration, I
        // poll seg_num[i] and seg_num[i+1] in a given iteration.
        let iter_limit = std::cmp::max(seg_nums.len() - 1, 0);
        Self {
            data: input,
            line_num,
            seg_nums,
            byte_nums,
            iter_limit,
            curr_seg_num: 0,
            remaining_graphemes: 0,
        }
    }

    /// A method that allows for the current TOMLSeg's state to be "frozen".
    /// Essentially, the iteration state is able to be recreated by calling
    /// ParserLine::next_seg.
    pub fn freeze(pline: Self, count: usize) -> Self {
        Self {
            remaining_graphemes: count,
            ..pline
        }
    }

    pub fn peek(&self) -> Option<TOMLSeg<'_>> {
        let remaining_graphs = self.remaining_graphemes;
        let cursors = &self.seg_nums;
        let bytes = &self.byte_nums;
        let mut output: Option<TOMLSeg<'_>> = None;
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
            let (byte_lb, byte_ub) = (bytes[curr_num], bytes[curr_num + 1]);
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            let num_elements = ub - lb;
            let skips = num_elements - remaining_graphs;
            let slice = &(self.data.as_str()[byte_lb..byte_ub]);

            let mut seg = TOMLSeg::new(slice);
            for _ in 0..skips {
                seg.next();
            }
            output = Some(seg);
        } else {
            let curr_num = self.curr_seg_num;
            if curr_num == self.iter_limit {
                // output is initialized to None already.
            } else {
                // produce full segment
                let (byte_lb, byte_ub) = (bytes[curr_num], bytes[curr_num + 1]);
                let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
                let slice = &(self.data.as_str()[byte_lb..byte_ub]);
                output = Some(TOMLSeg::new(slice));
            }
        }
        output
    }

    pub fn next_seg(&mut self) -> Option<TOMLSeg<'_>> {
        let remaining_graphs = self.remaining_graphemes;
        let cursors = &self.seg_nums;
        let bytes = &self.byte_nums;
        let mut output: Option<TOMLSeg<'_>> = None;
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
            let (byte_lb, byte_ub) = (bytes[curr_num], bytes[curr_num + 1]);
            let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
            let num_elements = ub - lb;
            let skips = num_elements - remaining_graphs;
            let slice = &(self.data.as_str()[byte_lb..byte_ub]);

            let mut seg = TOMLSeg::new(slice);
            for _ in 0..skips {
                seg.next();
            }
            output = Some(seg);
            self.remaining_graphemes = 0;
        } else {
            let curr_num = self.curr_seg_num;
            if curr_num == self.iter_limit {
                // output is initialized to None already.
            } else {
                // produce full segment
                let (byte_lb, byte_ub) = (bytes[curr_num], bytes[curr_num + 1]);
                let (lb, ub) = (cursors[curr_num], cursors[curr_num + 1]);
                let slice = &(self.data.as_str()[byte_lb..byte_ub]);
                output = Some(TOMLSeg::new(slice));
                self.curr_seg_num += 1;
            }
        }
        output
    }

    pub fn is_exhausted(&self) -> bool {
        self.curr_seg_num == self.iter_limit && self.remaining_graphemes == 0
    }

    pub fn line_num(&self) -> usize {
        self.line_num
    }

    /////////////////
    // Static Methods
    /////////////////

    pub fn replacement() -> Self {
        Self {
            data: String::new(),
            seg_nums: Vec::new(),
            byte_nums: Vec::new(),
            line_num: 0,
            iter_limit: 0,
            curr_seg_num: 0,
            remaining_graphemes: 0,
        }
    }

    /// Partitions the line into TOML-semantic segments.
    /// Given some
    ///     key = value # comment
    /// this function should find segments that represent the following split:
    ///     |key |=| value |# comment|
    fn find_segments(s: &str) -> (Vec<usize>, Vec<usize>) {
        let mut seg_nums: Vec<usize> = vec![];
        let mut byte_nums: Vec<usize> = vec![];
        let mut iter = s.grapheme_indices(true).peekable();

        let mut graphemes = 0;
        while let Some((byte_num, ch)) = iter.next() {
            match ch {
                COMMENT_TOKEN => {
                    byte_nums.push(byte_num);
                    seg_nums.push(graphemes);
                }
                KEY_VAL_SEP | SEQUENCE_DELIM => {
                    // push this delimiter's byte_num and the next grapheme's
                    // This is so the segment is only one grapheme long.
                    byte_nums.push(byte_num);
                    seg_nums.push(graphemes);

                    if iter.peek().is_none() {
                        panic!("<ParserLine::find_segments>: Premature EoF.");
                    }
                    let (next_offset, next_ch_peek) = iter.peek().unwrap();
                    byte_nums.push(*next_offset);
                    seg_nums.push(graphemes + 1);
                }
                _ => {
                    if graphemes == 0 {
                        byte_nums.push(0);
                        seg_nums.push(0);
                    }
                }
            }
            graphemes += 1;
        }

        // The segment numbers are monotonically increasing, so we can sort and remove duplicates
        // Duplicates arise in the case where characters of interest follow back to back (ex. "==")
        // These duplicates would result in empty segments if left in the vector.
        if !seg_nums.is_empty() {
            byte_nums.push(s.len());
            seg_nums.push(s.len()); // Add last range endpoint.
        }
        seg_nums.sort();
        seg_nums.dedup();
        byte_nums.sort();
        byte_nums.dedup();
        assert_eq!(byte_nums.len(), seg_nums.len());
        (seg_nums, byte_nums)
    }
}

mod tests {
    use super::ParserLine;
    /////////////
    // Functions
    /////////////

    pub fn main() {
        test_continuity();
        let s = "==#[]{}".to_string();

        print_segs(s.as_str());
        print_segs("some_key = value # this is a comment\n");
        print_segs("[\"This is a normal table key\"]\n");
        print_segs("{InlineTableKey: [An array, of, items]}\n");
        print_segs("a̐=é#ö̲\r\n");
    }

    fn print_segs(s: &str) {
        let mut segs = ParserLine::new(s.to_string(), 0);
        println!("\nSegments: {:?}", segs);
        while let Some(mut seg) = segs.next_seg() {
            println!("Segment: {:?}", seg);
            println!("Preview Char: {:?}", seg.peek());
            let vector = seg.collect::<Vec<_>>();
            println!("SegItem: {:?}", vector);
        }
        println!("\n");
    }
    
    fn test_continuity() {
        let mut pline = ParserLine::new("This is a test".to_string(), 0);
        let mut seg = pline.next_seg();
        println!("\n{:?}", seg);
        let mut seg = seg.unwrap();
        assert_eq!(Some("T"), seg.next());
        assert_eq!(Some(&"h"), seg.peek());
        let count = seg.count();

        pline = ParserLine::continuation(pline, count);
        seg = pline.next_seg().unwrap();
        assert_eq!("h", seg.next().unwrap());
    }
}