use super::UPPER_HEX;
use crate::extended::parser::UrlMode;

fn is_allowed(byte: u8, mode: UrlMode) -> bool {
    if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
        return true;
    }
    matches!(mode, UrlMode::Full)
        && matches!(
            byte,
            b'!' | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'/'
                | b'?'
                | b'#'
                | b'['
                | b']'
                | b'@'
        )
}

pub(crate) fn encode(input: &[u8], mode: UrlMode) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    for &byte in input {
        if is_allowed(byte, mode) {
            output.push(byte);
        } else {
            output.push(b'%');
            output.push(UPPER_HEX[usize::from(byte >> 4)]);
            output.push(UPPER_HEX[usize::from(byte & 0x0F)]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(input: &[u8]) -> Vec<u8> {
        encode(input, UrlMode::Component)
    }

    fn full(input: &[u8]) -> Vec<u8> {
        encode(input, UrlMode::Full)
    }

    #[test]
    fn component_leaves_unreserved_bytes_alone() {
        assert_eq!(component(b"AZaz09-._~"), b"AZaz09-._~");
    }

    #[test]
    fn component_encodes_reserved_characters() {
        assert_eq!(component(b"a b"), b"a%20b");
        assert_eq!(component(b"a/b"), b"a%2Fb");
        assert_eq!(component(b"a?b=c"), b"a%3Fb%3Dc");
        assert_eq!(component(b"%"), b"%25");
    }

    #[test]
    fn component_percent_encodes_utf8_bytes() {
        assert_eq!(component("é".as_bytes()), b"%C3%A9");
        assert_eq!(component(&[0xFF]), b"%FF");
    }

    #[test]
    fn full_keeps_url_structure() {
        assert_eq!(
            full(b"https://example.com/a b?q=1#f"),
            b"https://example.com/a%20b?q=1#f"
        );
        assert_eq!(full(b"mailto:a@b.com"), b"mailto:a@b.com");

        assert_eq!(full(b"a%20b"), b"a%2520b");
        assert_eq!(full(b"<>{}|\\^\""), b"%3C%3E%7B%7D%7C%5C%5E%22");
    }

    #[test]
    fn empty_input_encodes_to_empty() {
        assert_eq!(component(b""), b"");
        assert_eq!(full(b""), b"");
    }

    #[test]
    fn output_round_trips_through_a_decoder() {
        let input = b"a b/c?d=e&f=1#x";
        let encoded = component(input);
        assert_eq!(decode(&encoded), input);
    }

    fn decode(encoded: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();
        let mut index = 0;
        while index < encoded.len() {
            if encoded[index] == b'%' {
                let high = (encoded[index + 1] as char).to_digit(16).expect("hex");
                let low = (encoded[index + 2] as char).to_digit(16).expect("hex");
                output.push(u8::try_from(high * 16 + low).expect("byte"));
                index += 3;
            } else {
                output.push(encoded[index]);
                index += 1;
            }
        }
        output
    }
}
