use crate::prototype::constants;
use std::iter::{Peekable, Skip, Take};
use unicode_segmentation::{Graphemes, UnicodeSegmentation as utf8};

/// A struct for semantically segmenting a line for parsing.
/// The idea is to represent segment delimiters as column positions:
///     Ex. 1: |"This is a line representing a string." |# This is a comment. \n
///     Ex. 2: |key|=|simple_value|comment   // four segments
#[derive(Debug)]
pub struct ParserLine {
    line: String,
    segment_delims: Vec<usize>,
}

impl ParserLine {
    pub fn new() -> Self {
        ParserLine {
            line: String::with_capacity(100 * 5),
            segment_delims: Vec::new(),
        }
    }
    pub fn from(input: &str) -> Self {
        ParserLine {
            line: input.to_string(),
            segment_delims: Vec::new(),
        }
    }
    /// Results in an iterator over grapheme clusters
    pub fn graphemes(&self) -> Graphemes {
        utf8::graphemes(self.line.as_str(), true)
    }

    fn seg_delims(&self) -> &Vec<usize> {
        &self.segment_delims
    }

    /// Produces an iterator over semantic line segments
    /// An iterator of iterators
    pub fn seg_iter(&self) -> PLIterator {
        PLIterator::new(self)
    }
}

/// A trait that supports segmenting a given line into semantic TOML regions. For example,
/// given a line such as
///     some_key = value # comment
/// the function should provide cursor values such that the line is portioned as:
///     |some_key |=| value |# comment|
/// The idea is that segmenting the line will allow for easier parsing by enabling
/// determination of a given TOML semantic block.
pub trait TOMLSegments {
    fn find_segments(&mut self);
}

impl TOMLSegments for ParserLine {
    fn find_segments(&mut self) {
        use constants::*;
        let graphs = self.graphemes();
        let mut segments: Vec<usize> = vec![];
        for (i, ch) in graphs.enumerate() {
            match ch {
                COMMENT_TOKEN
                | TABLE_OPEN_TOKEN
                | TABLE_CLOSE_TOKEN
                | INLINE_OPENTABLE_TOKEN
                | INLINE_CLOSETABLE_TOKEN => {
                    segments.push(i);
                }
                "=" => {
                    segments.push(i);
                    segments.push(i + 1);
                }
                _ => {
                    if i == 0 {
                        segments.push(i);
                    }
                }
            }
        }
        segments.push(self.line.len());
        self.segment_delims = segments;
    }
}

impl<'a> IntoIterator for &'a ParserLine {
    type Item = Peekable<Skip<Take<Graphemes<'a>>>>;
    type IntoIter = PLIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PLIterator::new(self)
    }
}

pub struct PLIterator<'a> {
    curr_delim_num: usize,
    limit: usize,
    pline: &'a ParserLine,
}

impl<'a> PLIterator<'a> {
    fn new(pline: &'a ParserLine) -> Self {
        use std::cmp;
        let limit: usize = cmp::max(pline.seg_delims().len() - 1, 0);
        Self {
            curr_delim_num: 0,
            limit,
            pline,
        }
    }
}

impl<'a> Iterator for PLIterator<'a> {
    type Item = Peekable<Skip<Take<Graphemes<'a>>>>;
    fn next(&mut self) -> Option<Self::Item> {
        let curr_num = self.curr_delim_num;
        let delims = &self.pline.seg_delims();
        if curr_num == self.limit {
            None
        } else {
            let (lb, ub) = (delims[curr_num], delims[curr_num + 1]);
            let itr = self.pline.graphemes().take(ub).skip(lb).peekable();
            self.curr_delim_num += 1;
            Some(itr)
        }
    }
}
