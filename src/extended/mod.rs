pub(crate) mod align;
pub(crate) mod case;
pub(crate) mod color;
pub(crate) mod encoders;
pub(crate) mod limits;
pub(crate) mod number;
pub(crate) mod parser;
pub(crate) mod repeat;

#[cfg(test)]
mod property_tests;

use std::io::Write;

use crate::output::{Output, OutputError};
use crate::{DispatchError, PROGRAM, VERSION};

use color::ColorEnvironment;
use encoders::Encoder;
use parser::{Align, CaseMode, ColorWhen, Parsed, ResolvedOption, Style};

const HELP: &str = "\
Usage: pico --extended [OPTION]... [--] [STRING]...
Echo the STRING(s) to standard output using the extended feature set.

Options:
      --align=WHERE     align within --width; WHERE is left, right, or centre
                        (requires --width; default left)
      --width=N         pad to at least N Unicode scalar values
      --truncate        truncate to --width before padding (requires --width)
      --repeat=COUNT    repeat the text COUNT times (default separator empty)
      --separator=TEXT  join repetitions with TEXT (requires --repeat)
      --number-lines    prefix each line with its number and a TAB
      --hex             encode as lower-case hexadecimal
      --base64=VARIANT  base64 encode; VARIANT is standard, standard-nopad,
                        urlsafe, or urlsafe-nopad
      --json            encode as a quoted JSON string
      --url=MODE        percent-encode; MODE is component or full
      --case=MODE       convert case; MODE is upper, lower, ascii-upper, or
                        ascii-lower
      --color=WHEN      colour output; WHEN is auto, always, or never
      --style=NAME      colour style; NAME is success, error, warning,
                        information, muted, or emphasis (implies --color=auto)
      --help            display this help and exit (sole argument)
      --version         output version information and exit (sole argument)

Transformations are applied in a fixed order: case, then at most one encoder,
then repetition, then truncation, then padding/alignment, then line numbering,
then the colour wrapper, then one trailing newline. The encoders are mutually
exclusive.

Colour: auto colours only when standard output is a terminal and NO_COLOR is
unset or empty; an explicit --color=always overrides NO_COLOR. Escape sequences
are never encoded and never counted towards --width.

The first argument must be exactly --extended; PICO_COMPAT=1 disables extended
mode entirely. Options may appear before or after strings; use -- to end
option parsing. --help and --version are recognised only as the sole argument.
Widths count Unicode scalar values, not terminal display columns.
";

pub(crate) fn run<S, W>(
    args: &[S],
    out: &mut Output<W>,
    color_env: ColorEnvironment,
) -> Result<(), DispatchError>
where
    S: AsRef<[u8]>,
    W: Write,
{
    match parser::parse(args).map_err(DispatchError::Usage)? {
        Parsed::Help => out.write_all(HELP.as_bytes()).map_err(DispatchError::from),
        Parsed::Version => {
            let line = format!("{PROGRAM} {VERSION}\n");
            out.write_all(line.as_bytes()).map_err(DispatchError::from)
        }
        Parsed::Run(request) => execute(&request, out, color_env),
    }
}

struct Plan<'a> {
    case_mode: Option<CaseMode>,

    encoder: Option<Encoder>,

    repeat: Option<(u64, &'a [u8])>,

    width: Option<u64>,

    truncate: bool,

    align: Align,

    number_lines: bool,

    styling: bool,

    style: Style,
}

impl Plan<'_> {
    fn transforms(&self) -> bool {
        self.case_mode.is_some()
            || self.encoder.is_some()
            || self.repeat.is_some()
            || self.width.is_some()
            || self.number_lines
            || self.styling
    }
}

fn resolve<'a>(request: &parser::Request<'a>, color_env: ColorEnvironment) -> Plan<'a> {
    let mut plan = Plan {
        case_mode: None,
        encoder: None,
        repeat: None,
        width: None,
        truncate: false,
        align: Align::Left,
        number_lines: false,
        styling: false,
        style: color::DEFAULT_STYLE,
    };
    let mut separator: &[u8] = b"";
    let mut repeat_count: Option<u64> = None;
    let mut color_when: Option<ColorWhen> = None;
    let mut style_given = false;

    for option in &request.options {
        match *option {
            ResolvedOption::Case(mode) => plan.case_mode = Some(mode),
            ResolvedOption::Hex => plan.encoder = Some(Encoder::Hex),
            ResolvedOption::Base64(variant) => plan.encoder = Some(Encoder::Base64(variant)),
            ResolvedOption::Json => plan.encoder = Some(Encoder::Json),
            ResolvedOption::Url(mode) => plan.encoder = Some(Encoder::Url(mode)),
            ResolvedOption::Width(value) => plan.width = Some(value),
            ResolvedOption::Truncate => plan.truncate = true,
            ResolvedOption::Align(value) => plan.align = value,
            ResolvedOption::Repeat(count) => repeat_count = Some(count),
            ResolvedOption::Separator(text) => separator = text,
            ResolvedOption::NumberLines => plan.number_lines = true,
            ResolvedOption::Color(when) => color_when = Some(when),
            ResolvedOption::Style(style) => {
                plan.style = style;
                style_given = true;
            }
        }
    }
    plan.repeat = repeat_count.map(|count| (count, separator));

    plan.styling = match color_when {
        Some(when) => color::enabled(when, color_env),
        None => style_given && color::enabled(ColorWhen::Auto, color_env),
    };
    plan
}

fn transform(plan: &Plan<'_>, operands: &[&[u8]]) -> Result<Vec<u8>, DispatchError> {
    let mut text = join_operands(operands);

    if let Some(mode) = plan.case_mode {
        text = case::convert(&text, mode);
    }
    if let Some(encoder) = plan.encoder {
        text = encoders::encode(&text, encoder);
    }
    if let Some((count, separator)) = plan.repeat {
        let total = repeat::length(text.len(), separator.len(), count);
        match total {
            Some(total) if limits::check_total([total]).is_ok() => {
                text = repeat::apply(&text, separator, count);
            }
            _ => return Err(DispatchError::Usage(limits::too_large())),
        }
    }

    Ok(text)
}

fn execute<W: Write>(
    request: &parser::Request<'_>,
    out: &mut Output<W>,
    color_env: ColorEnvironment,
) -> Result<(), DispatchError> {
    let plan = resolve(request, color_env);

    if !plan.transforms() {
        return write_operands(&request.operands, out).map_err(DispatchError::from);
    }

    let text = transform(&plan, &request.operands)?;

    let (left, body, right) = if let Some(width) = plan.width {
        let body = if plan.truncate {
            align::truncate(&text, width)
        } else {
            text.as_slice()
        };
        let units = align::count_units(body);
        let (left, right) = align::padding(width, units, plan.align);
        (left, body, right)
    } else {
        (0, text.as_slice(), 0)
    };

    let numbering_bytes = if plan.number_lines {
        number::prefix_bytes(u64::try_from(number::split_lines(body).len()).unwrap_or(u64::MAX))
    } else {
        0
    };

    let prefix: &[u8] = if plan.styling {
        color::prefix(plan.style)
    } else {
        b""
    };
    let reset: &[u8] = if plan.styling { color::RESET } else { b"" };

    limits::check_total([
        u64::try_from(prefix.len()).unwrap_or(u64::MAX),
        left,
        u64::try_from(body.len()).unwrap_or(u64::MAX),
        numbering_bytes,
        right,
        u64::try_from(reset.len()).unwrap_or(u64::MAX),
        1,
    ])
    .map_err(DispatchError::Usage)?;

    let numbered = plan.number_lines.then(|| number::apply(body, left, right));

    out.write_all(prefix).map_err(DispatchError::from)?;
    if let Some(content) = numbered {
        out.write_all(&content).map_err(DispatchError::from)?;
    } else {
        write_spaces(out, left).map_err(DispatchError::from)?;
        out.write_all(body).map_err(DispatchError::from)?;
        write_spaces(out, right).map_err(DispatchError::from)?;
    }
    out.write_all(reset).map_err(DispatchError::from)?;
    out.write_all(b"\n").map_err(DispatchError::from)
}

fn join_operands(operands: &[&[u8]]) -> Vec<u8> {
    let mut capacity: usize = operands.iter().map(|operand| operand.len()).sum();
    capacity = capacity.saturating_add(operands.len().saturating_sub(1));

    let mut text = Vec::with_capacity(capacity);
    for (index, operand) in operands.iter().enumerate() {
        if index > 0 {
            text.push(b' ');
        }
        text.extend_from_slice(operand);
    }
    text
}

fn write_spaces<W: Write>(out: &mut Output<W>, mut count: u64) -> Result<(), OutputError> {
    const SPACES: [u8; 64] = [b' '; 64];

    while count >= 64 {
        out.write_all(&SPACES)?;
        count -= 64;
    }
    if count > 0 {
        let length = usize::try_from(count).unwrap_or(0);
        out.write_all(&SPACES[..length])?;
    }
    Ok(())
}

fn write_operands<W: Write>(operands: &[&[u8]], out: &mut Output<W>) -> Result<(), OutputError> {
    for (index, operand) in operands.iter().enumerate() {
        if index > 0 {
            out.write_all(b" ")?;
        }
        out.write_all(operand)?;
    }

    out.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Output;

    fn render(args: &[&[u8]]) -> Result<Vec<u8>, DispatchError> {
        render_env(
            args,
            ColorEnvironment {
                no_color: false,
                stdout_is_terminal: false,
            },
        )
    }

    fn render_env(args: &[&[u8]], color_env: ColorEnvironment) -> Result<Vec<u8>, DispatchError> {
        let mut sink = Vec::new();
        {
            let mut out = Output::new(&mut sink);
            run(args, &mut out, color_env)?;
            out.flush().map_err(DispatchError::Output)?;
        }
        Ok(sink)
    }

    #[test]
    fn operands_are_joined_with_spaces_and_a_newline() {
        assert_eq!(
            render(&[b"hello", b"world"]).expect("run"),
            b"hello world\n"
        );
    }

    #[test]
    fn no_operands_writes_only_a_newline() {
        assert_eq!(render(&[]).expect("run"), b"\n");
    }

    #[test]
    fn empty_operands_are_preserved() {
        assert_eq!(render(&[b"", b"x", b""]).expect("run"), b" x \n");
    }

    #[test]
    fn operands_keep_non_utf8_bytes() {
        assert_eq!(render(&[b"\xFF\x00mid"]).expect("run"), b"\xFF\x00mid\n");
    }

    #[test]
    fn the_end_of_options_marker_is_not_output() {
        assert_eq!(render(&[b"--", b"-n"]).expect("run"), b"-n\n");
    }

    #[test]
    fn help_and_version_are_written_to_the_output() {
        assert!(
            render(&[b"--help"])
                .expect("help")
                .starts_with(b"Usage: pico --extended")
        );
        assert_eq!(
            render(&[b"--version"]).expect("version"),
            format!("{PROGRAM} {VERSION}\n").into_bytes()
        );
    }

    #[test]
    fn a_rejected_request_writes_nothing() {
        let error = render(&[b"--width=100000000", b"x"]).expect_err("rejected");
        assert!(matches!(error, DispatchError::Usage(_)));
    }

    #[test]
    fn repetition_runs_after_encoding_and_before_padding() {
        assert_eq!(
            render(&[b"--base64=standard", b"--repeat=2", b"--separator=-", b"hi"]).expect("run"),
            b"aGk=-aGk=\n"
        );

        assert_eq!(
            render(&[b"--repeat=3", b"--separator=-", b"--width=4", b"ab"]).expect("run"),
            b"ab-ab-ab\n"
        );

        assert_eq!(
            render(&[
                b"--repeat=3",
                b"--separator=-",
                b"--width=5",
                b"--truncate",
                b"ab"
            ])
            .expect("run"),
            b"ab-ab\n"
        );
    }

    #[test]
    fn repetition_count_zero_and_one() {
        assert_eq!(render(&[b"--repeat=0", b"hi"]).expect("run"), b"\n");
        assert_eq!(render(&[b"--repeat=1", b"hi"]).expect("run"), b"hi\n");
    }

    #[test]
    fn line_numbering_runs_after_padding() {
        assert_eq!(
            render(&[b"--number-lines", b"a\nb"]).expect("run"),
            b"1\ta\n2\tb\n"
        );

        assert_eq!(
            render(&[b"--number-lines", b"--width=4", b"hi"]).expect("run"),
            b"1\thi  \n"
        );
    }

    #[test]
    fn colour_wraps_the_whole_line_in_always_mode() {
        let colored = ColorEnvironment {
            no_color: false,
            stdout_is_terminal: false,
        };
        assert_eq!(
            render_env(&[b"--color=always", b"--style=error", b"hi"], colored).expect("run"),
            b"\x1b[31mhi\x1b[0m\n"
        );

        assert_eq!(
            render(&[b"--color=auto", b"--style=error", b"hi"]).expect("run"),
            b"hi\n"
        );

        let terminal = ColorEnvironment {
            no_color: false,
            stdout_is_terminal: true,
        };
        assert_eq!(
            render_env(&[b"--style=success", b"hi"], terminal).expect("run"),
            b"\x1b[32mhi\x1b[0m\n"
        );
    }

    #[test]
    fn the_pipeline_applies_case_before_encoding() {
        assert_eq!(
            render(&[b"--case=upper", b"--hex", b"hi"]).expect("run"),
            b"4849\n"
        );
    }

    #[test]
    fn width_pads_and_aligns_the_encoded_text() {
        assert_eq!(
            render(&[b"--hex", b"--width=6", b"--align=right", b"hi"]).expect("run"),
            b"  6869\n"
        );
    }

    #[test]
    fn truncation_happens_before_padding() {
        assert_eq!(
            render(&[b"--width=3", b"--truncate", b"abcdef"]).expect("run"),
            b"abc\n"
        );
        assert_eq!(
            render(&[b"--width=5", b"--truncate", b"--align=right", b"hi"]).expect("run"),
            b"   hi\n"
        );
    }

    #[test]
    fn json_wraps_and_escapes() {
        assert_eq!(
            render(&[b"--json", br#"a"b"#]).expect("run"),
            b"\"a\\\"b\"\n"
        );
    }
}
