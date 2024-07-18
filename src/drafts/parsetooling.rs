fn main() {
    tests::main();
}

//stdlib imports
use std::iter::{Peekable, Skip, Take};
// third-party imports
use unicode_segmentation::{Graphemes, UnicodeSegmentation as utf8};
// internal imports
use super::constants::{
    COMMENT_TOKEN, INLINETAB_CLOSE_TOKEN, INLINETAB_OPEN_TOKEN, KEY_VAL_SEP, LITERAL_STR_TOKEN,
    SEQUENCE_DELIM, STR_TOKEN, TABLE_CLOSE_TOKEN, TABLE_OPEN_TOKEN,
};

//////////////
// Struct Defs
//////////////
pub type TOMLSeg<'a> = Peekable<Skip<Take<Graphemes<'a>>>>;

#[derive(Debug)]
pub struct ParserLine {
    line_num: usize,
    data: String,
    // iteration things
    seg_nums: Vec<usize>,         // a vector of what is essentially cursor positions to denote segment ranges.
    iter_limit: usize,            // The iteration termination value
    curr_seg_num: usize,          // x: 0 <= x <= iter_limit;
    remaining_graphemes: usize,   // a tracker for reproducing a given segment with some offset.
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

    /// A method that allows for the transfer of parsing context.
    /// Essentially, the iteration state is able to be recreated.
    pub fn continuation(pline: Self, count: usize) -> Self {
        Self {
            remaining_graphemes: count,
            ..pline
        }
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

    pub fn is_exhausted(&self) -> bool {
        if self.curr_seg_num == self.iter_limit && self.remaining_graphemes == 0 {
            true
        } else {
            false
        }
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
                COMMENT_TOKEN
                | INLINETAB_CLOSE_TOKEN
                | INLINETAB_OPEN_TOKEN
                | STR_TOKEN
                | LITERAL_STR_TOKEN
                | SEQUENCE_DELIM => seg_spots.push(i),
                KEY_VAL_SEP => {
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
