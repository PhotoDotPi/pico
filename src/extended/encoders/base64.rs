use crate::extended::parser::Base64;

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn variant(variant: Base64) -> (&'static [u8; 64], bool) {
    match variant {
        Base64::Standard => (STANDARD, true),
        Base64::StandardNoPad => (STANDARD, false),
        Base64::UrlSafe => (URL_SAFE, true),
        Base64::UrlSafeNoPad => (URL_SAFE, false),
    }
}

pub(crate) fn encode(input: &[u8], variant_kind: Base64) -> Vec<u8> {
    let (alphabet, pad) = variant(variant_kind);

    let mut output = Vec::with_capacity(input.len().div_ceil(3).saturating_mul(4));

    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        output.push(alphabet[usize::from(first >> 2)]);
        output.push(alphabet[usize::from(((first & 0x03) << 4) | (second >> 4))]);
        if chunk.len() > 1 {
            output.push(alphabet[usize::from(((second & 0x0F) << 2) | (third >> 6))]);
        } else if pad {
            output.push(b'=');
        }
        if chunk.len() > 2 {
            output.push(alphabet[usize::from(third & 0x3F)]);
        } else if pad {
            output.push(b'=');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(encoded: &[u8], url_safe: bool) -> Vec<u8> {
        let alphabet = if url_safe { URL_SAFE } else { STANDARD };
        let mut values = Vec::new();
        for &byte in encoded {
            if byte == b'=' {
                continue;
            }
            let value = alphabet
                .iter()
                .position(|&candidate| candidate == byte)
                .unwrap_or_else(|| panic!("invalid base64 byte {byte:#x}"));
            values.push(u8::try_from(value).expect("6-bit value"));
        }

        let mut output = Vec::new();
        for group in values.chunks(4) {
            if let Some(&a) = group.first() {
                output.push((a << 2) | (group.get(1).copied().unwrap_or(0) >> 4));
            }
            if group.len() >= 3 {
                let b = group[1];
                let c = group[2];
                output.push((b << 4) | (c >> 2));
            }
            if group.len() >= 4 {
                let c = group[2];
                let d = group[3];
                output.push((c << 6) | d);
            }
        }
        output
    }

    #[test]
    fn matches_known_base64_vectors() {
        let cases: &[(&[u8], &[u8])] = &[
            (b"", b""),
            (b"f", b"Zg=="),
            (b"ab", b"YWI="),
            (b"foo", b"Zm9v"),
            (b"foob", b"Zm9vYg=="),
            (b"fooba", b"Zm9vYmE="),
            (b"foobar", b"Zm9vYmFy"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                encode(input, Base64::Standard),
                *expected,
                "input {input:?}"
            );
        }
    }

    #[test]
    fn no_padding_variants_omit_equals() {
        assert_eq!(encode(b"f", Base64::StandardNoPad), b"Zg");
        assert_eq!(encode(b"ab", Base64::StandardNoPad), b"YWI");
        assert_eq!(encode(b"foob", Base64::StandardNoPad), b"Zm9vYg");
    }

    #[test]
    fn url_safe_uses_the_url_alphabet() {
        assert_eq!(encode(&[0xFB, 0xFF], Base64::Standard), b"+/8=");
        assert_eq!(encode(&[0xFB, 0xFF], Base64::UrlSafe), b"-_8=");
        assert_eq!(encode(&[0xFB, 0xFF], Base64::UrlSafeNoPad), b"-_8");
    }

    #[test]
    fn empty_input_encodes_to_empty() {
        for kind in [
            Base64::Standard,
            Base64::StandardNoPad,
            Base64::UrlSafe,
            Base64::UrlSafeNoPad,
        ] {
            assert_eq!(encode(b"", kind), b"");
        }
    }

    #[test]
    fn output_round_trips_for_every_length() {
        let input: Vec<u8> = (0u8..=200).collect();
        for len in 0..=input.len() {
            let slice = &input[..len];
            for (kind, url_safe) in [
                (Base64::Standard, false),
                (Base64::StandardNoPad, false),
                (Base64::UrlSafe, true),
                (Base64::UrlSafeNoPad, true),
            ] {
                let encoded = encode(slice, kind);
                assert_eq!(decode(&encoded, url_safe), slice, "len {len}");
            }
        }
    }
}
