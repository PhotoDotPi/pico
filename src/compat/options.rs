const HELP_OPTION: &[u8] = b"--help";

const VERSION_OPTION: &[u8] = b"--version";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Echo,

    Help,

    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Parsed {
    pub(crate) action: Action,

    pub(crate) first_operand: usize,

    pub(crate) display_return: bool,

    pub(crate) escapes: bool,
}

impl Parsed {
    fn sole(action: Action) -> Self {
        Self {
            action,
            first_operand: 1,
            display_return: true,
            escapes: false,
        }
    }
}

pub(crate) fn parse<S: AsRef<[u8]>>(args: &[S], allow_options: bool) -> Parsed {
    if allow_options && args.len() == 1 {
        let only = args[0].as_ref();
        if only == HELP_OPTION {
            return Parsed::sole(Action::Help);
        }
        if only == VERSION_OPTION {
            return Parsed::sole(Action::Version);
        }
    }

    let mut display_return = true;
    let mut escapes = false;
    let mut first_operand = 0;

    if allow_options {
        while let Some(argument) = args.get(first_operand) {
            let argument = argument.as_ref();
            if argument.first() != Some(&b'-') {
                break;
            }

            let letters = &argument[1..];

            if letters.is_empty() {
                break;
            }

            if !letters
                .iter()
                .all(|letter| matches!(letter, b'e' | b'E' | b'n'))
            {
                break;
            }

            for &letter in letters {
                match letter {
                    b'n' => display_return = false,
                    b'e' => escapes = true,
                    b'E' => escapes = false,

                    _ => unreachable!("letters are validated to be one of e, E, n"),
                }
            }

            first_operand += 1;
        }
    }

    Parsed {
        action: Action::Echo,
        first_operand,
        display_return,
        escapes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&[u8]]) -> Parsed {
        parse(args, true)
    }

    fn echo(first_operand: usize, display_return: bool, escapes: bool) -> Parsed {
        Parsed {
            action: Action::Echo,
            first_operand,
            display_return,
            escapes,
        }
    }

    #[test]
    fn no_arguments_is_a_plain_echo() {
        assert_eq!(parse_args(&[]), echo(0, true, false));
    }

    #[test]
    fn plain_operands_are_operands() {
        assert_eq!(parse_args(&[b"hello"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"a", b"b"]), echo(0, true, false));
        assert_eq!(parse_args(&[b""]), echo(0, true, false));
    }

    #[test]
    fn minus_n_is_recognised_only_leading() {
        assert_eq!(parse_args(&[b"-n"]), echo(1, false, false));
        assert_eq!(parse_args(&[b"-n", b"hi"]), echo(1, false, false));
        assert_eq!(parse_args(&[b"-n", b"a", b"b"]), echo(1, false, false));
    }

    #[test]
    fn minus_e_and_minus_e_upper_are_recognised() {
        assert_eq!(parse_args(&[b"-e", b"hi"]), echo(1, true, true));
        assert_eq!(parse_args(&[b"-E", b"hi"]), echo(1, true, false));
    }

    #[test]
    fn last_case_letter_wins() {
        assert_eq!(parse_args(&[b"-eE", b"hi"]), echo(1, true, false));
        assert_eq!(parse_args(&[b"-Ee", b"hi"]), echo(1, true, true));
    }

    #[test]
    fn clusters_are_supported() {
        assert_eq!(parse_args(&[b"-ne", b"hi"]), echo(1, false, true));
        assert_eq!(parse_args(&[b"-en", b"hi"]), echo(1, false, true));
        assert_eq!(parse_args(&[b"-nE", b"hi"]), echo(1, false, false));
        assert_eq!(parse_args(&[b"-nn", b"hi"]), echo(1, false, false));
        assert_eq!(parse_args(&[b"-enE", b"hi"]), echo(1, false, false));
    }

    #[test]
    fn multiple_separate_options_are_consumed() {
        assert_eq!(parse_args(&[b"-n", b"-e"]), echo(2, false, true));
        assert_eq!(parse_args(&[b"-n", b"-e", b"hi"]), echo(2, false, true));
        assert_eq!(parse_args(&[b"-e", b"-E", b"hi"]), echo(2, true, false));
    }

    #[test]
    fn unknown_letters_make_the_whole_argument_an_operand() {
        assert_eq!(parse_args(&[b"-x", b"hi"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-nex", b"hi"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-abc"]), echo(0, true, false));

        assert_eq!(parse_args(&[b"-enx", b"-n"]), echo(0, true, false));
    }

    #[test]
    fn lone_dash_is_an_operand() {
        assert_eq!(parse_args(&[b"-"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-", b"-n"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-n", b"-"]), echo(1, false, false));
    }

    #[test]
    fn double_dash_is_an_operand_and_stops_parsing() {
        assert_eq!(parse_args(&[b"--"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"--", b"-n"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-n", b"--", b"x"]), echo(1, false, false));
    }

    #[test]
    fn parsing_stops_at_the_first_operand() {
        assert_eq!(parse_args(&[b"x", b"-n"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-n", b"x", b"-e"]), echo(1, false, false));
        assert_eq!(parse_args(&[b"-e", b"x", b"-n"]), echo(1, true, true));
    }

    #[test]
    fn help_and_version_only_as_the_sole_argument() {
        assert_eq!(parse_args(&[b"--help"]).action, Action::Help);
        assert_eq!(parse_args(&[b"--version"]).action, Action::Version);

        assert_eq!(parse_args(&[b"--help", b"x"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"--version", b"x"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"--help", b"--version"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"x", b"--help"]), echo(0, true, false));
        assert_eq!(parse_args(&[b"-n", b"--help"]), echo(1, false, false));
    }

    #[test]
    fn disallowing_options_treats_everything_as_operands() {
        let parsed = parse(&[b"-n", b"hi"], false);
        assert_eq!(parsed, echo(0, true, false));

        let parsed = parse(&[b"--help"], false);
        assert_eq!(parsed, echo(0, true, false));
    }
}
