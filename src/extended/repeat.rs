pub(crate) fn length(text_len: usize, separator_len: usize, count: u64) -> Option<u64> {
    if count == 0 {
        return Some(0);
    }
    if text_len == 0 && separator_len == 0 {
        return Some(0);
    }

    let text = u64::try_from(text_len).ok()?;
    let separator = u64::try_from(separator_len).ok()?;

    let bodies = text.checked_mul(count)?;
    let separators = separator.checked_mul(count - 1)?;
    bodies.checked_add(separators)
}

pub(crate) fn apply(text: &[u8], separator: &[u8], count: u64) -> Vec<u8> {
    if count == 0 || (text.is_empty() && separator.is_empty()) {
        return Vec::new();
    }

    let total = length(text.len(), separator.len(), count)
        .and_then(|total| usize::try_from(total).ok())
        .unwrap_or(0);
    let mut output = Vec::with_capacity(total);

    for copy in 0..count {
        if copy > 0 {
            output.extend_from_slice(separator);
        }
        output.extend_from_slice(text);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_copies_is_empty() {
        assert_eq!(length(5, 3, 0), Some(0));
        assert_eq!(apply(b"hello", b"-", 0), b"");
    }

    #[test]
    fn one_copy_is_the_body_once() {
        assert_eq!(length(5, 3, 1), Some(5));
        assert_eq!(apply(b"hello", b"-", 1), b"hello");
    }

    #[test]
    fn copies_are_joined_by_the_separator() {
        assert_eq!(apply(b"ab", b"-", 3), b"ab-ab-ab");
        assert_eq!(apply(b"ab", b"", 3), b"ababab");
        assert_eq!(apply(b"ab", b"--", 2), b"ab--ab");
    }

    #[test]
    fn length_matches_the_materialised_output() {
        for (text, separator, count) in [
            (&b"x"[..], &b"-"[..], 5u64),
            (&b""[..], &b", "[..], 4),
            (&b"abc"[..], &b""[..], 3),
            (&b"a"[..], &b"bb"[..], 1),
        ] {
            let expected = length(text.len(), separator.len(), count).expect("length");
            assert_eq!(apply(text, separator, count).len() as u64, expected);
        }
    }

    #[test]
    fn empty_body_repeats_only_the_separator() {
        assert_eq!(apply(b"", b"-", 4), b"---");
        assert_eq!(apply(b"", b"-", 1), b"");
        assert_eq!(apply(b"", b"", 1_000_000_000), b"");
    }

    #[test]
    fn a_huge_count_of_nothing_is_empty_not_a_hang() {
        assert_eq!(length(0, 0, u64::MAX), Some(0));
        assert_eq!(apply(b"", b"", u64::MAX), b"");
    }

    #[test]
    fn overflowing_length_is_none() {
        assert_eq!(length(2, 0, u64::MAX), None);
        assert_eq!(length(usize::MAX, 0, 2), None);
    }
}
