use super::LOWER_HEX;

pub(crate) fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len().saturating_add(2));
    output.push(b'"');

    for &byte in input {
        match byte {
            b'"' => output.extend_from_slice(b"\\\""),
            b'\\' => output.extend_from_slice(b"\\\\"),
            0x08 => output.extend_from_slice(b"\\b"),
            0x09 => output.extend_from_slice(b"\\t"),
            0x0A => output.extend_from_slice(b"\\n"),
            0x0C => output.extend_from_slice(b"\\f"),
            0x0D => output.extend_from_slice(b"\\r"),
            0x00..=0x1F => {
                output.extend_from_slice(b"\\u00");
                output.push(LOWER_HEX[usize::from(byte >> 4)]);
                output.push(LOWER_HEX[usize::from(byte & 0x0F)]);
            }
            _ => output.push(byte),
        }
    }

    output.push(b'"');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_plain_text_in_quotes() {
        assert_eq!(encode(b"foo"), b"\"foo\"");
    }

    #[test]
    fn empty_input_is_an_empty_string() {
        assert_eq!(encode(b""), b"\"\"");
    }

    #[test]
    fn escapes_quotes_and_backslash() {
        assert_eq!(encode(br#"a"b"#), br#""a\"b""#);
        assert_eq!(encode(br"a\b"), br#""a\\b""#);
    }

    #[test]
    fn uses_short_escapes_for_common_controls() {
        assert_eq!(encode(b"\x08"), b"\"\\b\"");
        assert_eq!(encode(b"\x09"), b"\"\\t\"");
        assert_eq!(encode(b"\x0A"), b"\"\\n\"");
        assert_eq!(encode(b"\x0C"), b"\"\\f\"");
        assert_eq!(encode(b"\x0D"), b"\"\\r\"");
    }

    #[test]
    fn other_controls_use_unicode_escapes() {
        assert_eq!(encode(&[0x00]), b"\"\\u0000\"");
        assert_eq!(encode(&[0x01]), b"\"\\u0001\"");
        assert_eq!(encode(&[0x1F]), b"\"\\u001f\"");
    }

    #[test]
    fn non_ascii_bytes_pass_through() {
        assert_eq!(encode("é".as_bytes()), "\"é\"".as_bytes());

        assert_eq!(encode(&[0x7F]), b"\"\x7F\"");
    }

    #[test]
    fn invalid_bytes_are_preserved_verbatim() {
        assert_eq!(encode(&[0xFF]), &[b'"', 0xFF, b'"']);
    }
}
