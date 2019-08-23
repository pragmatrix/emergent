//! Line breaking algorithms.

use std::iter;

/// Breaks a string into lines by splitting at \n characters.
///
/// The implementation is symmetric in the sense that the returned lines
/// concatenated with `\n` will result the same string.
///
/// This also means, that the `\r` character will _always_ be treated as
/// whitespace and that no unicode line breaks are supported.
pub fn text_as_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut next = Some(0);

    let f = move || {
        if let Some(current) = next {
            match text[current..].find("\n") {
                Some(len) => {
                    let end = current + len;
                    let r = &text[current..end];
                    next = Some(end + 1);
                    Some(r)
                }
                None => {
                    let r = Some(&text[current..text.len()]);
                    next = None;
                    r
                }
            }
        } else {
            None
        }
    };

    iter::from_fn(f)
}

#[cfg(test)]
mod tests {
    #[test]
    fn single() {
        assert_eq!(tal("single"), vec!["single"])
    }

    #[test]
    fn empty() {
        assert_eq!(tal(""), vec![""])
    }

    #[test]
    fn two() {
        assert_eq!(tal("one\ntwo"), vec!["one", "two"])
    }

    #[test]
    fn two_nl() {
        assert_eq!(tal("one\ntwo\n"), vec!["one", "two", ""])
    }

    #[test]
    fn nl_one_nl() {
        assert_eq!(tal("\none\n"), vec!["", "one", ""])
    }

    fn tal(text: &str) -> Vec<&str> {
        super::text_as_lines(text).collect()
    }
}
