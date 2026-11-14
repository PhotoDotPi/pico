use std::io::Write;

use crate::output::{Output, OutputError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Complete,

    Stopped,
}

pub(crate) fn write_operands<S, W>(
    operands: &[S],
    display_return: bool,
    out: &mut Output<W>,
) -> Result<(), OutputError>
where
    S: AsRef<[u8]>,
    W: Write,
{
    let mut flow = Flow::Complete;

    for (index, operand) in operands.iter().enumerate() {
        if index > 0 {
            out.write_all(b" ")?;
        }
        if write_operand(operand.as_ref(), out)? == Flow::Stopped {
            flow = Flow::Stopped;
            break;
        }
    }

    if display_return && flow == Flow::Complete {
        out.write_all(b"\n")?;
    }

    Ok(())
}

fn write_operand<W: Write>(operand: &[u8], out: &mut Output<W>) -> Result<Flow, OutputError> {
    let mut index = 0;

    let mut run_start = 0;

    while index < operand.len() {
        if operand[index] != b'\\' || index + 1 >= operand.len() {
            index += 1;
            continue;
        }

        if run_start < index {
            out.write_all(&operand[run_start..index])?;
        }

        let escape = operand[index + 1];
        index += 2;

        match escape {
            b'a' => out.write_all(&[0x07])?,
            b'b' => out.write_all(&[0x08])?,
            b'c' => return Ok(Flow::Stopped),
            b'e' => out.write_all(&[0x1B])?,
            b'f' => out.write_all(&[0x0C])?,
            b'n' => out.write_all(&[0x0A])?,
            b'r' => out.write_all(&[0x0D])?,
            b't' => out.write_all(&[0x09])?,
            b'v' => out.write_all(&[0x0B])?,
            b'\\' => out.write_all(b"\\")?,
            b'x' => match hex_digit(operand.get(index)) {
                Some(high) => {
                    index += 1;
                    let value = match hex_digit(operand.get(index)) {
                        Some(low) => {
                            index += 1;

                            high * 16 + low
                        }
                        None => high,
                    };
                    out.write_all(&[value])?;
                }

                None => out.write_all(b"\\x")?,
            },
            b'0'..=b'7' => {
                let value = octal_value(escape, operand, &mut index);
                out.write_all(&[value])?;
            }

            other => out.write_all(&[b'\\', other])?,
        }

        run_start = index;
    }

    if run_start < index {
        out.write_all(&operand[run_start..index])?;
    }

    Ok(Flow::Complete)
}

#[allow(clippy::cast_possible_truncation)]
fn octal_value(escape: u8, operand: &[u8], index: &mut usize) -> u8 {
    let mut value: u32 = match escape {
        b'0' => match octal_digit(operand.get(*index)) {
            Some(digit) => {
                *index += 1;
                u32::from(digit)
            }

            None => return 0,
        },

        _ => u32::from(escape - b'0'),
    };

    let mut remaining = 2;
    while remaining > 0 {
        match octal_digit(operand.get(*index)) {
            Some(digit) => {
                value = value * 8 + u32::from(digit);
                *index += 1;
                remaining -= 1;
            }
            None => break,
        }
    }

    value as u8
}

fn octal_digit(byte: Option<&u8>) -> Option<u8> {
    match byte {
        Some(&digit @ b'0'..=b'7') => Some(digit - b'0'),
        _ => None,
    }
}

fn hex_digit(byte: Option<&u8>) -> Option<u8> {
    match byte {
        Some(&digit @ b'0'..=b'9') => Some(digit - b'0'),
        Some(&letter @ b'a'..=b'f') => Some(letter - b'a' + 10),
        Some(&letter @ b'A'..=b'F') => Some(letter - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Output;

    fn render(operands: &[&[u8]]) -> Vec<u8> {
        render_with(operands, true)
    }

    fn render_with(operands: &[&[u8]], display_return: bool) -> Vec<u8> {
        let mut sink = Vec::new();
        {
            let mut out = Output::new(&mut sink);
            write_operands(operands, display_return, &mut out).expect("write operands");
            out.flush().expect("flush");
        }
        sink
    }

    #[test]
    fn backslash_escape_is_a_single_backslash() {
        assert_eq!(render(&[br"\\"]), b"\\\n");

        assert_eq!(render(&[br"\\\\"]), b"\\\\\n");
    }

    #[test]
    fn single_byte_escapes_map_to_their_control_bytes() {
        let cases: &[(&[u8], u8)] = &[
            (br"\a", 0x07),
            (br"\b", 0x08),
            (br"\f", 0x0C),
            (br"\n", 0x0A),
            (br"\r", 0x0D),
            (br"\t", 0x09),
            (br"\v", 0x0B),
            (br"\e", 0x1B),
        ];

        for (operand, byte) in cases {
            assert_eq!(
                render(&[operand]),
                vec![*byte, b'\n'],
                "operand {:?}",
                String::from_utf8_lossy(operand)
            );
        }
    }

    #[test]
    fn adjacent_escapes_are_interpreted_independently() {
        assert_eq!(render(&[br"a\tb\nc"]), b"a\tb\nc\n");
        assert_eq!(render(&[br"\\\t\\"]), b"\\\t\\\n");
    }

    #[test]
    fn backslash_c_stops_all_output_and_the_newline() {
        assert_eq!(render(&[br"a\cb"]), b"a");
        assert_eq!(render(&[br"\c"]), b"");
        assert_eq!(render_with(&[br"x\c"], false), b"x");
    }

    #[test]
    fn backslash_c_suppresses_later_operands_and_separators() {
        assert_eq!(render(&[br"a\c", b"b"]), b"a");

        assert_eq!(render(&[b"a", br"b\cc", b"d"]), b"a b");

        assert_eq!(render(&[b"", br"\c"]), b" ");
    }

    #[test]
    fn leading_zero_octal_forms() {
        assert_eq!(render(&[br"\0"]), &[0x00, b'\n'][..]);
        assert_eq!(render(&[br"\0101"]), b"A\n");
        assert_eq!(render(&[br"\0123"]), b"S\n");
        assert_eq!(render(&[br"\0"]), b"\x00\n");

        assert_eq!(render(&[br"\01011"]), b"A1\n");
    }

    #[test]
    fn leading_zero_followed_by_non_octal_is_a_nul() {
        assert_eq!(render(&[br"\08"]), b"\x008\n");
    }

    #[test]
    fn octal_forms_without_leading_zero() {
        assert_eq!(render(&[br"\1"]), &[0x01, b'\n'][..]);
        assert_eq!(render(&[br"\12"]), &[0x0A, b'\n'][..]);
        assert_eq!(render(&[br"\101"]), b"A\n");
        assert_eq!(render(&[br"\1011"]), b"A1\n");
        assert_eq!(render(&[br"\777"]), &[0xFF, b'\n'][..]);
    }

    #[test]
    fn eight_is_not_octal_and_is_an_unknown_escape() {
        assert_eq!(render(&[br"\8"]), b"\\8\n");
        assert_eq!(render(&[br"\9"]), b"\\9\n");
    }

    #[test]
    fn hexadecimal_forms() {
        assert_eq!(render(&[br"\x41"]), b"A\n");
        assert_eq!(render(&[br"\x4"]), &[0x04, b'\n'][..]);
        assert_eq!(render(&[br"\x4A"]), b"J\n");
        assert_eq!(render(&[br"\xff"]), &[0xFF, b'\n'][..]);
        assert_eq!(render(&[br"\xFF"]), &[0xFF, b'\n'][..]);
        assert_eq!(render(&[br"\x0g"]), b"\x00g\n");
        assert_eq!(render(&[br"\x414"]), b"A4\n");
    }

    #[test]
    fn hex_without_digits_is_literal() {
        assert_eq!(render(&[br"\x"]), b"\\x\n");
        assert_eq!(render(&[br"\xg"]), b"\\xg\n");
        assert_eq!(render(&[br"\xZZ"]), b"\\xZZ\n");
    }

    #[test]
    fn unknown_escapes_keep_the_backslash_and_byte() {
        assert_eq!(render(&[br"\q"]), b"\\q\n");
        assert_eq!(render(&[br"a\qz"]), b"a\\qz\n");
        assert_eq!(render(&[br"\ "]), b"\\ \n");
        assert_eq!(render(&[br"\-"]), b"\\-\n");
    }

    #[test]
    fn trailing_backslash_is_literal() {
        assert_eq!(render(&[br"a\"]), b"a\\\n");
        assert_eq!(render(&[br"\\"]), b"\\\n");
        assert_eq!(render(&[br"abc\\"]), b"abc\\\n");
    }

    #[test]
    fn octal_ninth_bit_is_ignored() {
        assert_eq!(render(&[br"\777"]), &[0xFF, b'\n'][..]);
        assert_eq!(render(&[br"\400"]), &[0x00, b'\n'][..]);
        assert_eq!(render(&[br"\0777"]), &[0xFF, b'\n'][..]);
        assert_eq!(render(&[br"\0400"]), &[0x00, b'\n'][..]);
    }

    #[test]
    fn operands_without_escapes_pass_through_unchanged() {
        assert_eq!(render(&[b"plain"]), b"plain\n");
        assert_eq!(render(&[b"a b"]), b"a b\n");
        assert_eq!(render(&[]), b"\n");
        assert_eq!(render(&[b""]), b"\n");
        assert_eq!(render(&[b"", b"x", b""]), b" x \n");
    }

    #[test]
    fn non_utf8_bytes_survive_escape_interpretation() {
        assert_eq!(
            render(&[&[0xFF, b'\\', b't', 0xFE]]),
            &[0xFF, 0x09, 0xFE, b'\n'][..]
        );
    }

    #[test]
    fn nul_bytes_survive_through_the_output_layer() {
        assert_eq!(render(&[br"a\000b"]), &[b'a', 0x00, b'b', b'\n'][..]);
        assert_eq!(render(&[br"\x00"]), &[0x00, b'\n'][..]);
    }

    #[test]
    fn combined_sequence_in_a_single_operand() {
        assert_eq!(
            render(&[br"one\ttwo\nthree\x21\041"]),
            b"one\ttwo\nthree!!\n"
        );
    }
}
