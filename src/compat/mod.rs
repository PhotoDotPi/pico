pub(crate) mod escapes;
pub(crate) mod options;

use std::io::Write;

use crate::output::{Output, OutputError};
use crate::{HELP, PROGRAM, VERSION};

pub(crate) fn run<S, W>(
    args: &[S],
    posixly_correct: bool,
    out: &mut Output<W>,
) -> Result<(), OutputError>
where
    S: AsRef<[u8]>,
    W: Write,
{
    let parsed = options::parse(args, allow_options(posixly_correct, args));

    match parsed.action {
        options::Action::Help => out.write_all(HELP.as_bytes()),
        options::Action::Version => {
            let line = format!("{PROGRAM} {VERSION}\n");
            out.write_all(line.as_bytes())
        }
        options::Action::Echo => {
            let operands = &args[parsed.first_operand..];

            if parsed.escapes || posixly_correct {
                escapes::write_operands(operands, parsed.display_return, out)
            } else {
                write_literally(operands, parsed.display_return, out)
            }
        }
    }
}

fn allow_options<S: AsRef<[u8]>>(posixly_correct: bool, args: &[S]) -> bool {
    if !posixly_correct {
        return true;
    }
    args.first().is_some_and(|arg| arg.as_ref() == b"-n")
}

fn write_literally<S, W>(
    operands: &[S],
    display_return: bool,
    out: &mut Output<W>,
) -> Result<(), OutputError>
where
    S: AsRef<[u8]>,
    W: Write,
{
    for (index, operand) in operands.iter().enumerate() {
        if index > 0 {
            out.write_all(b" ")?;
        }
        out.write_all(operand.as_ref())?;
    }

    if display_return {
        out.write_all(b"\n")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Output;

    fn render(args: &[&[u8]]) -> Vec<u8> {
        render_posix(args, false)
    }

    fn render_posix(args: &[&[u8]], posixly_correct: bool) -> Vec<u8> {
        let mut sink = Vec::new();
        {
            let mut out = Output::new(&mut sink);
            run(args, posixly_correct, &mut out).expect("compatibility run");
            out.flush().expect("flush");
        }
        sink
    }

    #[test]
    fn joins_operands_with_single_spaces() {
        assert_eq!(
            render(&[b"a".as_slice(), b"".as_slice(), b"c".as_slice()]),
            b"a  c\n"
        );
    }

    #[test]
    fn empty_argument_list_writes_only_a_newline() {
        assert_eq!(render(&[]), b"\n");
    }

    #[test]
    fn suppressed_newline_still_separates_operands() {
        assert_eq!(
            render(&[b"-n".as_slice(), b"a".as_slice(), b"b".as_slice()]),
            b"a b"
        );
    }

    #[test]
    fn minus_e_interprets_escapes() {
        assert_eq!(render(&[b"-e".as_slice(), br"a\tb"]), b"a\tb\n");
    }

    #[test]
    fn minus_e_upper_keeps_escapes_literal() {
        assert_eq!(render(&[b"-E".as_slice(), br"a\tb"]), b"a\\tb\n");

        assert_eq!(render(&[br"a\tb"]), b"a\\tb\n");
    }

    #[test]
    fn cluster_with_trailing_e_interprets_escapes() {
        assert_eq!(render(&[b"-nEe".as_slice(), br"a\tb"]), b"a\tb");
    }

    #[test]
    fn sole_help_prints_usage() {
        assert!(render(&[b"--help".as_slice()]).starts_with(b"Usage: pico"));
    }

    #[test]
    fn sole_version_prints_program_and_version() {
        let text = String::from_utf8(render(&[b"--version".as_slice()])).expect("utf8");
        assert_eq!(text, format!("{PROGRAM} {VERSION}\n"));
    }

    #[test]
    fn posixly_correct_enables_escapes_by_default() {
        assert_eq!(render_posix(&[br"a\tb"], true), b"a\tb\n");

        assert_eq!(render_posix(&[br"-E", br"a\tb"], true), b"-E a\tb\n");
    }

    #[test]
    fn posixly_correct_makes_leading_options_data() {
        assert_eq!(render_posix(&[b"-e", br"a\tb"], true), b"-e a\tb\n");
        assert_eq!(render_posix(&[b"-n", b"hi"], true), b"hi");
        assert_eq!(render_posix(&[b"-x", b"hi"], true), b"-x hi\n");
        assert_eq!(render_posix(&[b"--help"], true), b"--help\n");
    }

    #[test]
    fn posixly_correct_honours_a_leading_minus_n_only() {
        assert_eq!(render_posix(&[b"-n", b"-e", br"a\tb"], true), b"a\tb");
        assert_eq!(render_posix(&[b"-n", b"-E", br"a\tb"], true), b"a\tb");

        assert_eq!(render_posix(&[b"-e", b"-n", b"hi"], true), b"-e -n hi\n");
    }

    #[test]
    fn posixly_correct_empty_value_is_still_set() {
        assert_eq!(render_posix(&[br"a\nb"], true), b"a\nb\n");
    }
}
