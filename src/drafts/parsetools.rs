#![allow(unused_imports)]
//stdlib imports
use std::iter::Peekable;
// third-party imports
use unicode_segmentation::{GraphemeIndices, Graphemes, UnicodeSegmentation as utf8};
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

    pub fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                None => break,
                Some(ch) => match *ch {
                    " " | "\t" => {
                        self.next();
                    }
                    _ => break,
                },
            }
        }
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
    data: String,    // the current line
    line_num: usize, // line's file location
    // iteration things
    seg_nums: Vec<usize>, // a vector of what is essentially cursor positions to denote segment ranges.
    byte_nums: Vec<usize>, // a vector of byte offsets to enable segment slice construction
    iter_limit: usize,    // The iteration terminal value
    curr_seg_num: usize,  // x: 0 <= x <= iter_limit;
    remaining_graphemes: usize, // a tracker for reproducing a given segment with some offset.
}
impl ParserLine {
    pub fn new(input: String, line_num: usize) -> Self {
        let (seg_nums, byte_nums) = Self::find_segments(input.as_str());
        // the iter_limit is set to 1 less than the number of elements in
        // the seg_num vector. This is because in the actual iteration, I
        // poll seg_num[i] and seg_num[i+1] in a given iteration.
        let iter_limit = {
            if seg_nums.is_empty() {
                0
            } else {
                seg_nums.len() - 1
            }
        };
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
                    let (next_offset, _) = iter.peek().unwrap();
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

/// A struct for creating paths through some graph-like structure
/// based on a provided delimiter.
#[derive(Clone, Debug)]
pub struct TPath<'a> {
    delimiter: &'a str,
    content: String,
}
/*  ==TPath Implementation==
 *  Traits:
 *      - PartialEq, Eq
 *      - IntoIterator
 *      - Display
*/
impl<'a> TPath<'a> {
    pub fn new(segments: Vec<String>, delimiter: &'a str) -> Option<Self> {
        if segments.is_empty() {
            None
        } else {
            let num_delimiters = segments.len() - 1;
            let mut content = String::new();
            for s in segments.iter().take(num_delimiters) {
                content.push_str(s.as_str());
                content.push_str(delimiter);
            }
            content.push_str(segments[num_delimiters].as_str());
            Some(Self { delimiter, content })
        }
    }

    /// Outputs the first segment of the path
    pub fn first(&self) -> &str {
        // if this method can be called, then at least one item exists
        self.into_iter().next().unwrap()
    }

    /// Outputs the last component of the path
    pub fn last(&self) -> &str {
        // if this method can be called, then at least one item exists
        self.into_iter().last().unwrap()
    }
}
impl<'a> PartialEq for TPath<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}
impl<'delim> Eq for TPath<'delim> {}
impl<'delim, 'a> IntoIterator for &'a TPath<'delim> {
    type Item = &'a str;
    type IntoIter = std::str::Split<'a, &'delim str>;
    fn into_iter(self) -> Self::IntoIter {
        self.content.split(self.delimiter)
    }
}
impl<'delim> std::fmt::Display for TPath<'delim> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::{ParserLine, TPath};

    /////////////
    // Functions
    /////////////

    #[test]
    fn tpath_instantiation() {
        let x = vec!["", "home", "tj", "documents"];
        let x: Vec<String> = x.iter().map(|x| x.to_string()).collect();

        assert!(TPath::new(x, "\0").is_some());
        assert_eq!(TPath::new(vec![], "/"), None);
    }

    #[test]
    fn tpath_eq() {
        let x = vec!["", "home", "tj", "documents"];
        let x: Vec<String> = x.iter().map(|x| x.to_string()).collect();
        let path = TPath::new(x.clone(), "\0");

        assert_eq!(path, path);
        assert_eq!(path, TPath::new(x.clone(), "\0"));
        assert_ne!(path, TPath::new(x.clone(), "/"));
    }

    #[test]
    fn test_continuity() {
        let mut pline = ParserLine::new("This is a test".to_string(), 0);
        let seg = pline.next_seg();
        println!("\n{:?}", seg);
        let mut seg = seg.unwrap();
        assert_eq!(Some("T"), seg.next());
        assert_eq!(Some(&"h"), seg.peek());
        let count = seg.count();

        pline = ParserLine::freeze(pline, count);
        seg = pline.next_seg().unwrap();
        assert_eq!("h", seg.next().unwrap());
    }

    #[test]
    fn test_blank_pline() {
        let mut blank = ParserLine::new("".to_string(), 0);
        assert_eq!(blank.next_seg().is_none(), true);
    }
}
