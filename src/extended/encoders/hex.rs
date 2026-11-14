use super::LOWER_HEX;

pub(crate) fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len().saturating_mul(2));
    for &byte in input {
        output.push(LOWER_HEX[usize::from(byte >> 4)]);
        output.push(LOWER_HEX[usize::from(byte & 0x0F)]);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_ascii_lowercase() {
        assert_eq!(encode(b"foo"), b"666f6f");
        assert_eq!(encode(b"Hello"), b"48656c6c6f");
    }

    #[test]
    fn empty_input_encodes_to_empty() {
        assert_eq!(encode(b""), b"");
    }

    #[test]
    fn encodes_all_byte_values() {
        let all: Vec<u8> = (0u8..=255).collect();
        let encoded = encode(&all);
        assert_eq!(encoded.len(), 512);
        assert_eq!(&encoded[..4], b"0001");

        assert_eq!(&encoded[508..], b"feff");
    }

    #[test]
    fn output_round_trips() {
        let input = b"\x00\x01\xFE\xFFhi";
        let encoded = encode(input);
        let decoded: Vec<u8> = encoded
            .chunks_exact(2)
            .map(|pair| {
                let high = (pair[0] as char).to_digit(16).expect("hex");
                let low = (pair[1] as char).to_digit(16).expect("hex");
                u8::try_from(high * 16 + low).expect("byte")
            })
            .collect();
        assert_eq!(decoded, input);
    }
}
