use crate::extended::parser::Align;

pub(crate) fn count_units(bytes: &[u8]) -> u64 {
    let mut units: u64 = 0;
    for chunk in bytes.utf8_chunks() {
        units += chunk.valid().chars().count() as u64;
        units += chunk.invalid().len() as u64;
    }
    units
}

pub(crate) fn truncate(bytes: &[u8], max_units: u64) -> &[u8] {
    let mut consumed: u64 = 0;
    let mut offset: usize = 0;

    for chunk in bytes.utf8_chunks() {
        let valid = chunk.valid();
        for (index, character) in valid.char_indices() {
            if consumed == max_units {
                return &bytes[..offset + index];
            }
            consumed += 1;
            debug_assert!(character.len_utf8() >= 1);
        }

        let invalid = chunk.invalid();
        for (index, _byte) in invalid.iter().enumerate() {
            if consumed == max_units {
                return &bytes[..offset + valid.len() + index];
            }
            consumed += 1;
        }

        offset += valid.len() + invalid.len();
    }

    bytes
}

pub(crate) fn padding(width: u64, units: u64, align: Align) -> (u64, u64) {
    let pad = width.saturating_sub(units);
    match align {
        Align::Left => (0, pad),
        Align::Right => (pad, 0),
        Align::Centre => {
            let left = pad / 2;
            (left, pad - left)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_counts_bytes() {
        assert_eq!(count_units(b""), 0);
        assert_eq!(count_units(b"hello"), 5);
        assert_eq!(count_units(b"a b"), 3);
    }

    #[test]
    fn one_scalar_is_one_unit_regardless_of_length() {
        assert_eq!(count_units("é".as_bytes()), 1);

        assert_eq!(count_units("日".as_bytes()), 1);
        assert_eq!(count_units("😀".as_bytes()), 1);

        assert_eq!(count_units("e\u{0301}".as_bytes()), 2);

        assert_eq!(count_units("👨‍👩‍👧".as_bytes()), 5);
    }

    #[test]
    fn invalid_bytes_count_one_unit_each() {
        assert_eq!(count_units(&[0xFF]), 1);
        assert_eq!(count_units(&[0xFF, 0xFE]), 2);

        assert_eq!(count_units("é".as_bytes()), 1);
        assert_eq!(count_units(&[b'a', 0xFF, b'b']), 3);

        assert_eq!(count_units(&[0xE2, 0x82]), 2);
    }

    #[test]
    fn truncation_keeps_scalars_whole() {
        assert_eq!(truncate(b"hello", 3), &b"hello"[..3]);
        assert_eq!(truncate(b"hello", 0), b"");
        assert_eq!(truncate(b"hello", 99), b"hello");

        assert_eq!(truncate("日本語".as_bytes(), 2), "日本".as_bytes());
        assert_eq!(truncate("日本語".as_bytes(), 1), "日".as_bytes());
        assert_eq!(truncate("日本語".as_bytes(), 0), b"");

        let truncated = truncate("日本".as_bytes(), 1);
        assert_eq!(truncated, "日".as_bytes());
    }

    #[test]
    fn truncation_handles_invalid_bytes_one_at_a_time() {
        let input = [b'a', 0xFF, 0xFE, b'b'];
        assert_eq!(truncate(&input, 2), &[b'a', 0xFF]);
        assert_eq!(truncate(&input, 1), b"a");
        assert_eq!(truncate(&input, 0), &[] as &[u8]);
        assert_eq!(truncate(&input, 4), &input);
    }

    #[test]
    fn padding_is_derived_from_alignment() {
        assert_eq!(padding(5, 2, Align::Left), (0, 3));
        assert_eq!(padding(5, 2, Align::Right), (3, 0));

        assert_eq!(padding(5, 2, Align::Centre), (1, 2));
        assert_eq!(padding(5, 1, Align::Centre), (2, 2));
        assert_eq!(padding(5, 4, Align::Centre), (0, 1));

        assert_eq!(padding(3, 3, Align::Left), (0, 0));
        assert_eq!(padding(3, 5, Align::Right), (0, 0));
    }
}
