use super::*;
use crate::output::Output;
use crate::testing::Rng;

use color::{ColorEnvironment, DEFAULT_STYLE, RESET};
use parser::Style;

fn render(args: &[&[u8]]) -> (Result<(), DispatchError>, Vec<u8>) {
    render_env(
        args,
        ColorEnvironment {
            no_color: false,
            stdout_is_terminal: false,
        },
    )
}

fn render_env(args: &[&[u8]], color_env: ColorEnvironment) -> (Result<(), DispatchError>, Vec<u8>) {
    let mut sink = Vec::new();
    let result = {
        let mut out = Output::new(&mut sink);
        let result = run(args, &mut out, color_env);
        let _ = out.flush();
        result
    };
    (result, sink)
}

const NEWLINE: u8 = b'\n';

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_hex(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() % 2 != 0 {
        return None;
    }
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        output.push(hex_digit(pair[0])? * 16 + hex_digit(pair[1])?);
    }
    Some(output)
}

fn decode_base64_standard(bytes: &[u8]) -> Option<Vec<u8>> {
    fn value(byte: u8) -> Option<u32> {
        match byte {
            b'A'..=b'Z' => Some(u32::from(byte - b'A')),
            b'a'..=b'z' => Some(u32::from(byte - b'a' + 26)),
            b'0'..=b'9' => Some(u32::from(byte - b'0' + 52)),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut unpadded = bytes;
    if let Some(stripped) = unpadded.strip_suffix(b"==") {
        unpadded = stripped;
    } else if let Some(stripped) = unpadded.strip_suffix(b"=") {
        unpadded = stripped;
    }

    let mut output = Vec::with_capacity(unpadded.len() * 3 / 4);
    let mut accumulator: u32 = 0;
    let mut bits = 0u32;
    for &byte in unpadded {
        accumulator = (accumulator << 6) | value(byte)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(u8::try_from((accumulator >> bits) & 0xFF).ok()?);
        }
    }
    Some(output)
}

fn percent_decode_component(bytes: &[u8]) -> Option<Vec<u8>> {
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            let high = hex(*bytes.get(index + 1)?)?;
            let low = hex(*bytes.get(index + 2)?)?;
            output.push(high * 16 + low);
            index += 3;
        } else {
            if !matches!(
                byte,
                b'-' | b'.' | b'_' | b'~' | b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            ) {
                return None;
            }
            output.push(byte);
            index += 1;
        }
    }
    Some(output)
}

#[test]
fn hex_round_trips_random_bytes() {
    let mut rng = Rng::new(0x4E1);
    for _ in 0..300 {
        let input = rng.bytes(64);
        let args: &[&[u8]] = &[b"--hex".as_slice(), b"--".as_slice(), input.as_slice()];
        let (result, output) = render(args);
        result.expect("hex run");
        assert_eq!(output.last(), Some(&NEWLINE), "trailing newline");
        let encoded = &output[..output.len() - 1];
        assert_eq!(encoded.len(), input.len() * 2, "two digits per byte");
        assert!(encoded.iter().all(u8::is_ascii_hexdigit), "hex digits only");
        assert_eq!(decode_hex(encoded).expect("decodable"), input);
    }
}

#[test]
fn base64_round_trips_random_bytes() {
    let mut rng = Rng::new(0xBA5E);
    for _ in 0..300 {
        let input = rng.bytes(64);
        let args: &[&[u8]] = &[
            b"--base64=standard".as_slice(),
            b"--".as_slice(),
            input.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("base64 run");
        let encoded = &output[..output.len() - 1];
        assert_eq!(decode_base64_standard(encoded).expect("decodable"), input);
    }
}

#[test]
fn url_component_encoding_round_trips_random_printable_text() {
    let mut rng = Rng::new(0x0398);
    for _ in 0..200 {
        let input = rng.printable(48);
        let args: &[&[u8]] = &[
            b"--url=component".as_slice(),
            b"--".as_slice(),
            input.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("url run");
        let encoded = &output[..output.len() - 1];
        assert_eq!(
            percent_decode_component(encoded).expect("decodable"),
            input,
            "component encoding is reversible over printable ASCII"
        );
    }
}

#[test]
fn repetition_length_matches_the_checked_arithmetic() {
    let mut rng = Rng::new(0x2E9A7);
    for _ in 0..200 {
        let text = rng.printable(16);
        let separator = rng.printable(4);
        let count = rng.below(50);
        let count_text = count.to_string();
        let args: &[&[u8]] = &[
            b"--repeat".as_slice(),
            count_text.as_bytes(),
            b"--separator".as_slice(),
            separator.as_slice(),
            b"--".as_slice(),
            text.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("repeat run");
        let expected = repeat::length(text.len(), separator.len(), count).expect("checked length");
        assert_eq!(output.len() as u64, expected + 1, "length + newline");
    }
}

#[test]
fn an_empty_repetition_of_nothing_is_empty_for_any_count() {
    let (result, output) = render(&[
        b"--repeat=18446744073709551615",
        b"--separator=",
        b"--",
        b"",
    ]);
    result.expect("empty repetition");
    assert_eq!(output, b"\n");
}

#[test]
fn an_oversized_or_overflowing_repetition_writes_nothing() {
    let mut rng = Rng::new(0x0FE1);
    for _ in 0..100 {
        let text = rng.printable(16);
        if text.is_empty() {
            continue;
        }
        let count = (rng.next_u64() | (1u64 << 32)).to_string();
        let args: &[&[u8]] = &[
            b"--repeat".as_slice(),
            count.as_bytes(),
            b"--".as_slice(),
            text.as_slice(),
        ];
        let (result, output) = render(args);
        assert!(
            matches!(result, Err(DispatchError::Usage(_))),
            "refused, not attempted"
        );
        assert!(output.is_empty(), "no partial output on refusal");
    }
}

#[test]
fn width_output_has_the_predicted_scalar_count() {
    let mut rng = Rng::new(0x71D7);
    for _ in 0..300 {
        let text = rng.text(24);
        let width = rng.below(65);
        let width_text = width.to_string();
        let input_units = std::str::from_utf8(&text)
            .expect("generated text is valid UTF-8")
            .chars()
            .count();
        let units = |output: &[u8]| {
            std::str::from_utf8(output)
                .expect("padding keeps valid UTF-8")
                .chars()
                .count()
        };

        let args: &[&[u8]] = &[
            b"--width".as_slice(),
            width_text.as_bytes(),
            text.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("width run");
        assert_eq!(
            units(&output[..output.len() - 1]),
            input_units.max(usize::try_from(width).unwrap_or(usize::MAX))
        );

        let args: &[&[u8]] = &[
            b"--width".as_slice(),
            width_text.as_bytes(),
            b"--truncate".as_slice(),
            text.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("truncate run");
        assert_eq!(
            units(&output[..output.len() - 1]),
            usize::try_from(width).unwrap_or(usize::MAX)
        );
    }
}

#[test]
fn numbering_prefixes_every_line_and_preserves_the_body() {
    let mut rng = Rng::new(0x0B17);
    for _ in 0..300 {
        let body = rng.text(24);
        let args: &[&[u8]] = &[b"--number-lines".as_slice(), body.as_slice()];
        let (result, output) = render(args);
        result.expect("numbering run");

        if body.is_empty() {
            assert_eq!(output, b"\n");
            continue;
        }

        let numbered = &output[..output.len() - 1];
        let mut stripped: Vec<&[u8]> = Vec::new();
        for (index, line) in numbered.split(|byte| *byte == b'\n').enumerate() {
            let prefix = format!("{}\t", index + 1);
            let content = line
                .strip_prefix(prefix.as_bytes())
                .unwrap_or_else(|| panic!("line {index} of {numbered:?} lacks its prefix"));
            stripped.push(content);
        }
        let joined = stripped.join(b"\n".as_slice());
        let expected: &[u8] = body.strip_suffix(b"\n").unwrap_or(&body);
        assert_eq!(joined, expected, "body preserved");
    }
}

#[test]
fn colour_wraps_the_whole_line_and_plain_output_never_contains_esc() {
    let styles = [
        (Style::Success, b"success".as_slice()),
        (Style::Error, b"error".as_slice()),
        (Style::Warning, b"warning".as_slice()),
        (Style::Information, b"information".as_slice()),
        (Style::Muted, b"muted".as_slice()),
        (Style::Emphasis, b"emphasis".as_slice()),
    ];
    let mut rng = Rng::new(0xC010);
    for _ in 0..100 {
        let text = rng.text(24);
        let (style, name) = rng.choice(&styles);

        let args: &[&[u8]] = &[
            b"--color=always".as_slice(),
            b"--style".as_slice(),
            name,
            text.as_slice(),
        ];
        let (result, output) = render(args);
        result.expect("colour run");
        let prefix = color::prefix(style);
        assert!(output.starts_with(prefix), "starts with the SGR prefix");
        assert!(output.ends_with([RESET, b"\n".as_slice()].concat().as_slice()));
        let middle = &output[prefix.len()..output.len() - RESET.len() - 1];
        assert!(!middle.contains(&0x1B), "the body carries no escapes");

        let args: &[&[u8]] = &[b"--color=always".as_slice(), text.as_slice()];
        let (result, output) = render(args);
        result.expect("default-style run");
        assert!(output.starts_with(color::prefix(DEFAULT_STYLE)));

        let args: &[&[u8]] = &[b"--color=auto".as_slice(), text.as_slice()];
        let (result, output) = render(args);
        result.expect("auto run");
        assert!(!output.contains(&0x1B), "no escapes when colour is off");
    }
}

#[test]
fn random_argument_lists_never_panic_and_never_partially_output() {
    const OPTIONS: &[&[u8]] = &[
        b"--hex",
        b"--json",
        b"--base64=standard",
        b"--url=component",
        b"--case=upper",
        b"--case=ascii-lower",
        b"--width=5",
        b"--width=0",
        b"--truncate",
        b"--align=centre",
        b"--align=right",
        b"--repeat=2",
        b"--repeat=0",
        b"--repeat=30000000",
        b"--separator=-",
        b"--number-lines",
        b"--color=always",
        b"--color=never",
        b"--color=auto",
        b"--style=muted",
        b"--style=error",
        b"--",
        b"-",
        b"--bogus",
        b"--repeat=-1",
        b"--hex=1",
    ];

    let mut rng = Rng::new(0xF022);
    for _ in 0..1500 {
        let count = usize::try_from(rng.below(7)).unwrap_or(0);
        let mut args: Vec<&[u8]> = Vec::with_capacity(count);
        let mut colour_always = false;
        for _ in 0..count {
            if rng.below(3) == 0 {
                let text: &'static [u8] = Box::leak(rng.printable(8).into_boxed_slice());
                args.push(text);
            } else {
                let option: &'static [u8] = rng.choice(OPTIONS);
                colour_always |= option == b"--color=always";
                args.push(option);
            }
        }

        let (result, output) = render(&args);
        match result {
            Ok(()) => {
                assert!(output.ends_with(b"\n"), "args {args:?}");
                if !colour_always {
                    assert!(
                        !output.contains(&0x1B),
                        "no escapes without --color=always: {args:?}"
                    );
                }
            }
            Err(DispatchError::Usage(_)) => {
                assert!(output.is_empty(), "no partial output for {args:?}");
            }
            Err(other) => panic!("unexpected failure for {args:?}: {other:?}"),
        }
    }
}
