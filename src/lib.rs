#![allow(dead_code, unused_variables, unused_imports)]
use unicode_segmentation::UnicodeSegmentation as utf8;

pub mod drafts;
pub mod prototype;

mod toml {
    use super::utf8;
    /////////////////////
    // Helper Function(s)
    /////////////////////
    /// Determines the presence of a substring in the provided reference.
    ///
    /// refstring: &[&str] A slice of str slices. Envisioned as &Vec<&str>.
    ///
    /// This function does not intend to search the entire refstring for the substring.
    /// It is solely concerned with whether the first `n` graphemes match where
    /// `n` is the number of graphemes in the target substring.
    ///
    /// # Examples
    ///
    /// ```
    /// use unicode_segmentation::UnicodeSegmentation as utf8;
    ///
    /// let test = "こんにちは！私の名前はTjです。";
    /// let vec = utf8::graphemes(test, true).collect::<Vec<&str>>();
    ///
    /// assert_eq!(found_sstr("こんこんTjで", &vec), false);
    /// assert_eq!(found_sstr("！私の名", &vec[5..]), true);
    /// assert_eq!(found_sstr("", &vec), true);
    /// ```
    fn found_sstr(sstr: &str, refstring: &[&str]) -> bool {
        let sstr_graphemes = utf8::graphemes(sstr, true).collect::<Vec<&str>>();
        let sstrlen = sstr_graphemes.len(); // Instead of # of bytes, use # of graphemes.
        if sstrlen > refstring.len() {
            return false;
        }

        for i in 0..sstrlen {
            println!("{}, {}", sstr_graphemes[i], refstring[i]);
            if sstr_graphemes[i] != refstring[i] {
                return false;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
