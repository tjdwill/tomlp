/// Testing parsing boolean values. Written in a manner to prevent extra allocation.
use std::iter::{Peekable, Skip, Take};
use unicode_segmentation::{UnicodeSegmentation as utf8, Graphemes}; // 1.11.0


fn main() {
    let test = "  false  \n";
    let tseg = test.graphemes(true).take(100).skip(0).peekable();
    println!("{:?}", parse_bool(test));
}

fn parse_bool(s: &str) -> Option<bool> {
    let mut seg = s
        .graphemes(true)
        .take(100)
        .skip(0)
        .peekable();
        
    skip_ws(&mut seg);
    match seg.peek() {
        None => None,
        Some(ch) => {
            match *ch {
                "t" => {
                    for c in "true".graphemes(true) {
                        match seg.next() {
                            Some(grapheme) => {
                                if grapheme != c {
                                    return None
                                }
                            }
                            None => return None
                        }
                    }
                    // exhaust iterator
                    loop {
                        match seg.next() {
                            Some(val) => {
                                match val {
                                    " " | "\t" | "\n" => continue,
                                    _ => return None
                                }
                            }
                            None => break
                        }
                    }
                    return Some(true)
                }
                "f" => {
                    for c in "false".graphemes(true) {
                        match seg.next() {
                            Some(grapheme) => {
                                if grapheme != c {
                                    return None
                                }
                            }
                            None => return None
                        }
                    }
                    // exhaust iterator
                    loop {
                        match seg.next() {
                            Some(val) => {
                                match val {
                                    " " | "\t" | "\n" => continue,
                                    _ => return None
                                }
                            }
                            None => break
                        }
                    }
                    return Some(false)
                }
                _ => return None
            }
        }
    }
}


fn skip_ws(seg: &mut Peekable<Skip<Take<Graphemes<'_>>>>) {
    loop {
        match seg.peek() {
            None => break,
            Some(ch) => {
                match *ch {
                    " " | "\t" | "\n" => {seg.next();},
                    _ => break
                }
            }
        }
    }
}
