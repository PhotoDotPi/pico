#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

mod compat;
mod env;
mod extended;
mod output;
mod platform;

#[cfg(test)]
mod testing;

use env::Env;
use extended::color::ColorEnvironment;
use extended::parser::UsageError;
use output::{Output, OutputError};

pub const PROGRAM: &str = "pico";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod exit {

    pub const SUCCESS: u8 = 0;

    pub const IO_ERROR: u8 = 1;

    pub const BROKEN_PIPE: u8 = 141;
}

const HELP: &str = "\
Usage: pico [SHORT-OPTION]... [STRING]...
  or:  pico LONG-OPTION
Echo the STRING(s) to standard output.

  -n     do not output the trailing newline
  -e     enable interpretation of backslash escapes
  -E     disable interpretation of backslash escapes (default)
      --extended   activate extended mode (see: pico --extended --help)

NOTE: pico is under development. Extended mode implements case conversion,
encoding (hex/base64/JSON/URL), width/alignment, repetition, line numbering, and
colour.
";

#[must_use]
pub fn run_from_env() -> ExitCode {
    let args = platform::args();
    let env = Env::from_process();
    let stdout = io::stdout();
    let stderr = io::stderr();

    let stdout_is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    ExitCode::from(run_with(&args, env, stdout_is_terminal, &mut out, &mut err))
}

#[must_use]
pub fn run(args: &[OsString], out: &mut impl Write) -> u8 {
    let env = Env::from_process();
    let mut err = io::sink();
    run_with(args, env, false, out, &mut err)
}

#[derive(Debug)]
enum DispatchError {
    Output(OutputError),

    Usage(UsageError),
}

impl From<OutputError> for DispatchError {
    fn from(error: OutputError) -> Self {
        Self::Output(error)
    }
}

fn run_with(
    args: &[OsString],
    env: Env,
    stdout_is_terminal: bool,
    out: &mut impl Write,
    err: &mut impl Write,
) -> u8 {
    let mut output = Output::new(out);
    let result = dispatch(args, env, stdout_is_terminal, &mut output);

    let outcome = match result {
        Ok(()) => output.flush(),
        Err(DispatchError::Output(error)) => Err(error),
        Err(DispatchError::Usage(error)) => {
            let _ = writeln!(err, "{PROGRAM}: {error}");
            return exit::IO_ERROR;
        }
    };

    match outcome {
        Ok(()) => exit::SUCCESS,
        Err(OutputError::BrokenPipe) => exit::BROKEN_PIPE,
        Err(OutputError::Other) => {
            let _ = writeln!(err, "{PROGRAM}: write error");
            exit::IO_ERROR
        }
    }
}

fn dispatch<W: Write>(
    args: &[OsString],
    env: Env,
    stdout_is_terminal: bool,
    out: &mut Output<W>,
) -> Result<(), DispatchError> {
    let arg_bytes = platform::arg_bytes(args);

    let extended_requested = !env.pico_compat
        && arg_bytes
            .first()
            .is_some_and(|arg| arg.as_ref() == b"--extended".as_slice());

    if extended_requested {
        let color_env = ColorEnvironment {
            no_color: env.no_color,
            stdout_is_terminal,
        };
        return extended::run(&arg_bytes[1..], out, color_env);
    }

    compat::run(&arg_bytes, env.posixly_correct, out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_case(args: &[&str], pico_compat: bool) -> (u8, Vec<u8>, Vec<u8>) {
        run_case_full(
            args,
            Env {
                pico_compat,
                posixly_correct: false,
                no_color: false,
            },
            false,
        )
    }

    fn run_case_env(
        args: &[&str],
        pico_compat: bool,
        posixly_correct: bool,
    ) -> (u8, Vec<u8>, Vec<u8>) {
        run_case_full(
            args,
            Env {
                pico_compat,
                posixly_correct,
                no_color: false,
            },
            false,
        )
    }

    fn run_case_full(args: &[&str], env: Env, stdout_is_terminal: bool) -> (u8, Vec<u8>, Vec<u8>) {
        let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_with(&os_args, env, stdout_is_terminal, &mut out, &mut err);
        (code, out, err)
    }

    #[test]
    fn plain_echo_joins_with_spaces_and_newline() {
        let (code, out, err) = run_case(&["hello", "world"], false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hello world\n");
        assert_eq!(err, b"");
    }

    #[test]
    fn minus_n_suppresses_the_newline() {
        let (code, out, err) = run_case(&["-n", "hi"], false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hi");
        assert_eq!(err, b"");
    }

    #[test]
    fn extended_acts_as_a_plain_echo() {
        let (code, out, err) = run_case(&["--extended", "hello", "world"], false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hello world\n");
        assert_eq!(err, b"");
    }

    #[test]
    fn extended_with_no_arguments_writes_a_newline() {
        let (code, out, err) = run_case(&["--extended"], false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"\n");
        assert_eq!(err, b"");
    }

    #[test]
    fn extended_unknown_option_is_a_usage_error() {
        let (code, out, err) = run_case(&["--extended", "--bogus"], false);
        assert_eq!(code, exit::IO_ERROR);
        assert_eq!(out, b"");
        assert_eq!(err, b"pico: unknown option '--bogus'\n");
    }

    #[test]
    fn extended_colour_modes_resolve_through_the_terminal_flag() {
        let plain = Env {
            pico_compat: false,
            posixly_correct: false,
            no_color: false,
        };

        let (code, out, err) = run_case_full(&["--extended", "--color=always", "hi"], plain, false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"\x1b[36mhi\x1b[0m\n");
        assert_eq!(err, b"");

        let (code, out, err) = run_case_full(&["--extended", "--color=never", "hi"], plain, false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hi\n");
        assert_eq!(err, b"");

        let (_, out, _) = run_case_full(&["--extended", "--color=auto", "hi"], plain, true);
        assert_eq!(out, b"\x1b[36mhi\x1b[0m\n");
        let (_, out, _) = run_case_full(&["--extended", "--color=auto", "hi"], plain, false);
        assert_eq!(out, b"hi\n");
    }

    #[test]
    fn extended_help_is_the_sole_argument() {
        let (code, out, err) = run_case(&["--extended", "--help"], false);
        assert_eq!(code, exit::SUCCESS);
        assert!(out.starts_with(b"Usage: pico --extended"));
        assert_eq!(err, b"");
    }

    #[test]
    fn pico_compat_forces_the_extended_string_as_data() {
        let (code, out, err) = run_case(&["--extended"], true);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"--extended\n");
        assert_eq!(err, b"");
    }

    #[test]
    fn extended_only_activates_in_the_first_position() {
        let (code, out, _err) = run_case(&["x", "--extended"], false);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"x --extended\n");
    }

    #[test]
    fn public_run_seam_reports_status_and_output() {
        let args = vec![OsString::from("-n"), OsString::from("hi")];
        let mut out = Vec::new();
        let code = run(&args, &mut out);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hi");
    }

    #[test]
    fn posixly_correct_enables_escapes_and_treats_options_as_data() {
        let (code, out, err) = run_case_env(&[r"a\tb"], false, true);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"a\tb\n");
        assert_eq!(err, b"");

        let (code, out, _err) = run_case_env(&["-e", r"a\tb"], false, true);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"-e a\tb\n");
    }

    #[test]
    fn posixly_correct_honours_a_leading_minus_n() {
        let (code, out, _err) = run_case_env(&["-n", "hi"], false, true);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(out, b"hi");
    }

    #[test]
    fn posixly_correct_does_not_change_success_status() {
        let (code, _out, err) = run_case_env(&[r"a\cb"], false, true);
        assert_eq!(code, exit::SUCCESS);
        assert_eq!(err, b"");
    }
}
