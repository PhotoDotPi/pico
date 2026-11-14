use crate::extended::parser::CaseMode;

pub(crate) fn convert(input: &[u8], mode: CaseMode) -> Vec<u8> {
    match mode {
        CaseMode::AsciiUpper => ascii(input, true),
        CaseMode::AsciiLower => ascii(input, false),
        CaseMode::Upper => unicode(input, true),
        CaseMode::Lower => unicode(input, false),
    }
}

fn ascii(input: &[u8], upper: bool) -> Vec<u8> {
    input
        .iter()
        .map(|&byte| {
            if upper {
                byte.to_ascii_uppercase()
            } else {
                byte.to_ascii_lowercase()
            }
        })
        .collect()
}

fn unicode(input: &[u8], upper: bool) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());

    for chunk in input.utf8_chunks() {
        let valid = chunk.valid();
        if upper {
            output.extend_from_slice(valid.to_uppercase().as_bytes());
        } else {
            output.extend_from_slice(valid.to_lowercase().as_bytes());
        }
        output.extend_from_slice(chunk.invalid());
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper(input: &[u8]) -> Vec<u8> {
        convert(input, CaseMode::Upper)
    }

    fn lower(input: &[u8]) -> Vec<u8> {
        convert(input, CaseMode::Lower)
    }

    #[test]
    fn ascii_modes_fold_only_ascii_letters() {
        assert_eq!(
            convert(b"Hello, World!", CaseMode::AsciiUpper),
            b"HELLO, WORLD!"
        );
        assert_eq!(
            convert(b"Hello, World!", CaseMode::AsciiLower),
            b"hello, world!"
        );

        assert_eq!(
            convert("é".as_bytes(), CaseMode::AsciiUpper),
            "é".as_bytes()
        );
        assert_eq!(convert(&[0xFF, b'a'], CaseMode::AsciiUpper), &[0xFF, b'A']);
    }

    #[test]
    fn unicode_modes_fold_multibyte_letters() {
        assert_eq!(upper("café".as_bytes()), "CAFÉ".as_bytes());
        assert_eq!(lower("CAFÉ".as_bytes()), "café".as_bytes());

        assert_eq!(upper("σ".as_bytes()), "Σ".as_bytes());
    }

    #[test]
    fn unicode_upper_handles_multi_character_expansions() {
        assert_eq!(upper("Straße".as_bytes()), "STRASSE".as_bytes());

        assert_eq!(upper("ﬁ".as_bytes()), "FI".as_bytes());
    }

    #[test]
    fn unicode_lower_handles_expansions_and_final_sigma() {
        assert_eq!(lower("İ".as_bytes()), "i\u{0307}".as_bytes());

        assert_eq!(lower("ΟΣ".as_bytes()), "ος".as_bytes());
    }

    #[test]
    fn invalid_bytes_are_preserved_verbatim() {
        assert_eq!(upper(&[0xFF, b'a', 0xFE]), &[0xFF, b'A', 0xFE]);

        assert_eq!(lower(&[0xC3]), &[0xC3]);
    }

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(upper(b""), b"");
        assert_eq!(convert(b"", CaseMode::AsciiLower), b"");
    }

    #[test]
    fn combining_sequences_are_case_folded_per_scalar() {
        assert_eq!(upper("e\u{0301}".as_bytes()), "E\u{0301}".as_bytes());
    }
}
