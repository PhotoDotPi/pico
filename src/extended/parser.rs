use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorWhen {
    Auto,

    Always,

    Never,
}

impl ColorWhen {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"auto" => Some(Self::Auto),
            b"always" => Some(Self::Always),
            b"never" => Some(Self::Never),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Style {
    Success,

    Error,

    Warning,

    Information,

    Muted,

    Emphasis,
}

impl Style {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"success" => Some(Self::Success),
            b"error" => Some(Self::Error),
            b"warning" => Some(Self::Warning),
            b"information" => Some(Self::Information),
            b"muted" => Some(Self::Muted),
            b"emphasis" => Some(Self::Emphasis),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Align {
    Left,

    Right,

    Centre,
}

impl Align {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"left" => Some(Self::Left),
            b"right" => Some(Self::Right),
            b"centre" => Some(Self::Centre),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Base64 {
    Standard,

    StandardNoPad,

    UrlSafe,

    UrlSafeNoPad,
}

impl Base64 {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"standard" => Some(Self::Standard),
            b"standard-nopad" => Some(Self::StandardNoPad),
            b"urlsafe" => Some(Self::UrlSafe),
            b"urlsafe-nopad" => Some(Self::UrlSafeNoPad),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlMode {
    Component,

    Full,
}

impl UrlMode {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"component" => Some(Self::Component),
            b"full" => Some(Self::Full),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaseMode {
    Upper,

    Lower,

    AsciiUpper,

    AsciiLower,
}

impl CaseMode {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"upper" => Some(Self::Upper),
            b"lower" => Some(Self::Lower),
            b"ascii-upper" => Some(Self::AsciiUpper),
            b"ascii-lower" => Some(Self::AsciiLower),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedOption<'a> {
    Color(ColorWhen),

    Style(Style),

    Repeat(u64),

    Separator(&'a [u8]),

    Align(Align),

    Width(u64),

    Truncate,

    Hex,

    Base64(Base64),

    Json,

    Url(UrlMode),

    Case(CaseMode),

    NumberLines,
}

impl ResolvedOption<'_> {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Color(_) => "color",
            Self::Style(_) => "style",
            Self::Repeat(_) => "repeat",
            Self::Separator(_) => "separator",
            Self::Align(_) => "align",
            Self::Width(_) => "width",
            Self::Truncate => "truncate",
            Self::Hex => "hex",
            Self::Base64(_) => "base64",
            Self::Json => "json",
            Self::Url(_) => "url",
            Self::Case(_) => "case",
            Self::NumberLines => "number-lines",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Request<'a> {
    pub(crate) options: Vec<ResolvedOption<'a>>,

    pub(crate) operands: Vec<&'a [u8]>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Parsed<'a> {
    Help,

    Version,

    Run(Request<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UsageError {
    UnknownOption(String),

    MissingValue(&'static str),

    UnexpectedValue(&'static str),

    InvalidValue {
        option: &'static str,

        value: String,

        expected: &'static str,
    },

    DuplicateOption(&'static str),

    MissingDependency {
        option: &'static str,

        requires: &'static str,
    },

    ConflictingOptions {
        option: &'static str,

        conflicts_with: &'static str,
    },

    OutputTooLarge {
        limit: u64,
    },
}

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOption(token) => write!(formatter, "unknown option '{token}'"),
            Self::MissingValue(option) => {
                write!(formatter, "option '--{option}' requires a value")
            }
            Self::UnexpectedValue(option) => {
                write!(formatter, "option '--{option}' does not take a value")
            }
            Self::InvalidValue {
                option,
                value,
                expected,
            } => write!(
                formatter,
                "invalid value '{value}' for option '--{option}' (expected {expected})"
            ),
            Self::DuplicateOption(option) => {
                write!(formatter, "option '--{option}' specified more than once")
            }
            Self::MissingDependency { option, requires } => {
                write!(
                    formatter,
                    "option '--{option}' requires option '--{requires}'"
                )
            }
            Self::ConflictingOptions {
                option,
                conflicts_with,
            } => write!(
                formatter,
                "option '--{option}' cannot be combined with '--{conflicts_with}'"
            ),
            Self::OutputTooLarge { limit } => {
                write!(
                    formatter,
                    "refusing to write more than {limit} bytes (resource limit)"
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arity {
    Flag,

    Value,
}

struct Spec {
    name: &'static str,

    arity: Arity,

    expected: &'static str,
}

const SPECS: &[Spec] = &[
    Spec {
        name: "color",
        arity: Arity::Value,
        expected: "one of: auto, always, never",
    },
    Spec {
        name: "style",
        arity: Arity::Value,
        expected: "one of: success, error, warning, information, muted, emphasis",
    },
    Spec {
        name: "repeat",
        arity: Arity::Value,
        expected: "a non-negative integer",
    },
    Spec {
        name: "separator",
        arity: Arity::Value,
        expected: "any text",
    },
    Spec {
        name: "align",
        arity: Arity::Value,
        expected: "one of: left, right, centre",
    },
    Spec {
        name: "width",
        arity: Arity::Value,
        expected: "a non-negative integer",
    },
    Spec {
        name: "truncate",
        arity: Arity::Flag,
        expected: "no value",
    },
    Spec {
        name: "hex",
        arity: Arity::Flag,
        expected: "no value",
    },
    Spec {
        name: "base64",
        arity: Arity::Value,
        expected: "one of: standard, standard-nopad, urlsafe, urlsafe-nopad",
    },
    Spec {
        name: "json",
        arity: Arity::Flag,
        expected: "no value",
    },
    Spec {
        name: "url",
        arity: Arity::Value,
        expected: "one of: component, full",
    },
    Spec {
        name: "case",
        arity: Arity::Value,
        expected: "one of: upper, lower, ascii-upper, ascii-lower",
    },
    Spec {
        name: "number-lines",
        arity: Arity::Flag,
        expected: "no value",
    },
];

const DEPENDENCIES: &[(&str, &str)] = &[
    ("separator", "repeat"),
    ("truncate", "width"),
    ("align", "width"),
];

const EXCLUSIVE_GROUPS: &[&[&str]] = &[&["hex", "base64", "json", "url"]];

pub(crate) fn parse<'a, S: AsRef<[u8]>>(args: &'a [S]) -> Result<Parsed<'a>, UsageError> {
    if let [only] = args {
        match only.as_ref() {
            b"--help" => return Ok(Parsed::Help),
            b"--version" => return Ok(Parsed::Version),
            _ => {}
        }
    }

    let mut options: Vec<ResolvedOption<'a>> = Vec::new();
    let mut operands: Vec<&'a [u8]> = Vec::new();
    let mut index = 0;
    let mut end_of_options = false;

    while let Some(argument) = args.get(index) {
        let argument = argument.as_ref();
        index += 1;

        if end_of_options || argument == b"-" || !argument.starts_with(b"-") {
            operands.push(argument);
            continue;
        }
        if argument == b"--" {
            end_of_options = true;
            continue;
        }

        let (name, inline) = split_option(argument)?;
        let spec = SPECS
            .iter()
            .find(|spec| spec.name.as_bytes() == name)
            .ok_or_else(|| UsageError::UnknownOption(format!("--{}", escape(name))))?;

        if options.iter().any(|existing| existing.name() == spec.name) {
            return Err(UsageError::DuplicateOption(spec.name));
        }

        let option = match spec.arity {
            Arity::Flag => {
                if inline.is_some() {
                    return Err(UsageError::UnexpectedValue(spec.name));
                }
                resolve_flag(spec)
            }
            Arity::Value => {
                let value = if let Some(value) = inline {
                    value
                } else {
                    let value = args.get(index).ok_or(UsageError::MissingValue(spec.name))?;
                    index += 1;
                    value.as_ref()
                };
                resolve_value(spec, value)?
            }
        };

        options.push(option);
    }

    validate_dependencies(&options)?;
    validate_conflicts(&options)?;

    Ok(Parsed::Run(Request { options, operands }))
}

fn split_option(argument: &[u8]) -> Result<(&[u8], Option<&[u8]>), UsageError> {
    let Some(body) = argument.strip_prefix(b"--") else {
        return Err(UsageError::UnknownOption(escape(argument)));
    };
    Ok(match body.iter().position(|byte| *byte == b'=') {
        Some(position) => (&body[..position], Some(&body[position + 1..])),
        None => (body, None),
    })
}

fn resolve_flag(spec: &Spec) -> ResolvedOption<'static> {
    match spec.name {
        "truncate" => ResolvedOption::Truncate,
        "hex" => ResolvedOption::Hex,
        "json" => ResolvedOption::Json,
        "number-lines" => ResolvedOption::NumberLines,

        other => unreachable!("{other} is not a flag option"),
    }
}

fn resolve_value<'a>(spec: &Spec, value: &'a [u8]) -> Result<ResolvedOption<'a>, UsageError> {
    let option = match spec.name {
        "color" => ResolvedOption::Color(parse_enum(value, spec, ColorWhen::from_bytes)?),
        "style" => ResolvedOption::Style(parse_enum(value, spec, Style::from_bytes)?),
        "align" => ResolvedOption::Align(parse_enum(value, spec, Align::from_bytes)?),
        "base64" => ResolvedOption::Base64(parse_enum(value, spec, Base64::from_bytes)?),
        "url" => ResolvedOption::Url(parse_enum(value, spec, UrlMode::from_bytes)?),
        "case" => ResolvedOption::Case(parse_enum(value, spec, CaseMode::from_bytes)?),
        "repeat" => ResolvedOption::Repeat(parse_count(value, spec)?),
        "width" => ResolvedOption::Width(parse_count(value, spec)?),
        "separator" => ResolvedOption::Separator(value),

        other => unreachable!("{other} does not take a value"),
    };
    Ok(option)
}

fn parse_enum<T>(
    value: &[u8],
    spec: &Spec,
    from_bytes: fn(&[u8]) -> Option<T>,
) -> Result<T, UsageError> {
    from_bytes(value).ok_or_else(|| invalid_value(spec, value))
}

fn parse_count(value: &[u8], spec: &Spec) -> Result<u64, UsageError> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(invalid_value(spec, value));
    }

    let mut count: u64 = 0;
    for &byte in value {
        let digit = u64::from(byte - b'0');
        count = count
            .checked_mul(10)
            .and_then(|total| total.checked_add(digit))
            .ok_or_else(|| invalid_value(spec, value))?;
    }

    Ok(count)
}

fn invalid_value(spec: &Spec, value: &[u8]) -> UsageError {
    UsageError::InvalidValue {
        option: spec.name,
        value: escape(value),
        expected: spec.expected,
    }
}

fn validate_dependencies(options: &[ResolvedOption<'_>]) -> Result<(), UsageError> {
    for option in options {
        let name = option.name();
        let Some(&(_, requires)) = DEPENDENCIES
            .iter()
            .find(|(dependent, _)| *dependent == name)
        else {
            continue;
        };
        if !options.iter().any(|other| other.name() == requires) {
            return Err(UsageError::MissingDependency {
                option: name,
                requires,
            });
        }
    }
    Ok(())
}

fn validate_conflicts(options: &[ResolvedOption<'_>]) -> Result<(), UsageError> {
    for group in EXCLUSIVE_GROUPS {
        let mut seen: Option<&'static str> = None;
        for option in options {
            let name = option.name();
            if !group.contains(&name) {
                continue;
            }
            if let Some(first) = seen {
                return Err(UsageError::ConflictingOptions {
                    option: name,
                    conflicts_with: first,
                });
            }
            seen = Some(name);
        }
    }
    Ok(())
}

fn escape(bytes: &[u8]) -> String {
    bytes.escape_ascii().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_bytes<'a>(args: &'a [&'a [u8]]) -> Result<Parsed<'a>, UsageError> {
        parse(args)
    }

    fn request<'a>(args: &'a [&'a [u8]]) -> Request<'a> {
        match parse_bytes(args) {
            Ok(Parsed::Run(request)) => request,
            other => panic!("expected a request, got {other:?}"),
        }
    }

    fn error(args: &[&[u8]]) -> UsageError {
        match parse_bytes(args) {
            Ok(parsed) => panic!("expected an error, got {parsed:?}"),
            Err(error) => error,
        }
    }

    #[test]
    fn no_arguments_is_an_empty_request() {
        assert_eq!(
            request(&[]),
            Request {
                options: Vec::new(),
                operands: Vec::new(),
            }
        );
    }

    #[test]
    fn operands_are_collected_verbatim() {
        let parsed = request(&[b"hello", b"", b"a b", b"-", b"\xFF\x00"]);
        assert_eq!(
            parsed,
            Request {
                options: Vec::new(),
                operands: vec![
                    b"hello".as_slice(),
                    b"".as_slice(),
                    b"a b".as_slice(),
                    b"-".as_slice(),
                    b"\xFF\x00".as_slice(),
                ],
            }
        );
    }

    #[test]
    fn double_dash_ends_options() {
        let parsed = request(&[b"--", b"--color=always", b"-n", b"--"]);
        assert_eq!(
            parsed,
            Request {
                options: Vec::new(),
                operands: vec![
                    b"--color=always".as_slice(),
                    b"-n".as_slice(),
                    b"--".as_slice(),
                ],
            }
        );
    }

    #[test]
    fn options_and_operands_may_interleave() {
        let parsed = request(&[b"a", b"--case=upper", b"b", b"--hex", b"c"]);
        assert_eq!(
            parsed.options,
            vec![ResolvedOption::Case(CaseMode::Upper), ResolvedOption::Hex]
        );
        assert_eq!(
            parsed.operands,
            vec![b"a".as_slice(), b"b".as_slice(), b"c".as_slice()]
        );
    }

    #[test]
    fn every_option_parses() {
        let parsed = request(&[
            b"--color=auto",
            b"--style=success",
            b"--repeat=3",
            b"--separator=,",
            b"--align=centre",
            b"--width=10",
            b"--truncate",
            b"--base64=standard-nopad",
            b"--case=ascii-lower",
            b"--number-lines",
        ]);
        assert_eq!(
            parsed.options,
            vec![
                ResolvedOption::Color(ColorWhen::Auto),
                ResolvedOption::Style(Style::Success),
                ResolvedOption::Repeat(3),
                ResolvedOption::Separator(b","),
                ResolvedOption::Align(Align::Centre),
                ResolvedOption::Width(10),
                ResolvedOption::Truncate,
                ResolvedOption::Base64(Base64::StandardNoPad),
                ResolvedOption::Case(CaseMode::AsciiLower),
                ResolvedOption::NumberLines,
            ]
        );
    }

    #[test]
    fn each_encoder_parses_on_its_own() {
        assert_eq!(request(&[b"--hex"]).options, vec![ResolvedOption::Hex]);
        assert_eq!(request(&[b"--json"]).options, vec![ResolvedOption::Json]);
        assert_eq!(
            request(&[b"--url=component"]).options,
            vec![ResolvedOption::Url(UrlMode::Component)]
        );
        assert_eq!(
            request(&[b"--base64=urlsafe"]).options,
            vec![ResolvedOption::Base64(Base64::UrlSafe)]
        );
    }

    #[test]
    fn values_may_be_attached_or_separate() {
        let attached = request(&[b"--color=always"]);
        let separate = request(&[b"--color", b"always"]);
        assert_eq!(attached.options, separate.options);
        assert_eq!(
            attached.options,
            vec![ResolvedOption::Color(ColorWhen::Always)]
        );
    }

    #[test]
    fn separate_value_consumes_the_next_argument_verbatim() {
        assert_eq!(
            error(&[b"--color", b"--hex"]),
            UsageError::InvalidValue {
                option: "color",
                value: "--hex".to_string(),
                expected: "one of: auto, always, never",
            }
        );

        assert_eq!(
            error(&[b"--style", b"--"]),
            UsageError::InvalidValue {
                option: "style",
                value: "--".to_string(),
                expected: "one of: success, error, warning, information, muted, emphasis",
            }
        );
    }

    #[test]
    fn empty_attached_value_is_rejected_for_an_enum() {
        assert_eq!(
            error(&[b"--color="]),
            UsageError::InvalidValue {
                option: "color",
                value: String::new(),
                expected: "one of: auto, always, never",
            }
        );
    }

    #[test]
    fn empty_separator_and_counts_are_accepted() {
        let parsed = request(&[b"--repeat=0", b"--separator="]);
        assert_eq!(
            parsed.options,
            vec![ResolvedOption::Repeat(0), ResolvedOption::Separator(b"")]
        );
    }

    #[test]
    fn a_missing_value_at_the_end_is_an_error() {
        assert_eq!(error(&[b"--color"]), UsageError::MissingValue("color"));
        assert_eq!(
            error(&[b"x", b"--repeat"]),
            UsageError::MissingValue("repeat")
        );
    }

    #[test]
    fn flags_reject_an_attached_value() {
        assert_eq!(error(&[b"--hex=1"]), UsageError::UnexpectedValue("hex"));
        assert_eq!(
            error(&[b"--number-lines=yes"]),
            UsageError::UnexpectedValue("number-lines")
        );
    }

    #[test]
    fn unknown_options_are_rejected() {
        assert_eq!(
            error(&[b"--bogus"]),
            UsageError::UnknownOption("--bogus".to_string())
        );
        assert_eq!(
            error(&[b"--bogus=1"]),
            UsageError::UnknownOption("--bogus".to_string())
        );

        assert_eq!(error(&[b"-n"]), UsageError::UnknownOption("-n".to_string()));
        assert_eq!(
            error(&[b"---x"]),
            UsageError::UnknownOption("---x".to_string())
        );
        assert_eq!(
            error(&[b"--=x"]),
            UsageError::UnknownOption("--".to_string())
        );
    }

    #[test]
    fn control_bytes_in_an_unknown_option_are_escaped() {
        assert_eq!(
            error(&[b"--b\x01g"]),
            UsageError::UnknownOption("--b\\x01g".to_string())
        );
    }

    #[test]
    fn invalid_enum_values_are_rejected() {
        assert_eq!(
            error(&[b"--align=center"]),
            UsageError::InvalidValue {
                option: "align",
                value: "center".to_string(),
                expected: "one of: left, right, centre",
            }
        );
        assert_eq!(
            error(&[b"--base64=url"]),
            UsageError::InvalidValue {
                option: "base64",
                value: "url".to_string(),
                expected: "one of: standard, standard-nopad, urlsafe, urlsafe-nopad",
            }
        );
    }

    #[test]
    fn counts_are_decimal_and_bounded() {
        assert_eq!(
            request(&[b"--repeat=007"]).options,
            vec![ResolvedOption::Repeat(7)]
        );
        assert_eq!(
            request(&[b"--width=18446744073709551615"]).options,
            vec![ResolvedOption::Width(u64::MAX)]
        );

        for value in [
            b"".as_slice(),
            b"-1",
            b"+3",
            b" 3",
            b"3 ",
            b"abc",
            b"18446744073709551616",
        ] {
            let args = [b"--repeat=".as_slice(), value].concat();
            assert_eq!(
                error(&[args.as_slice()]),
                UsageError::InvalidValue {
                    option: "repeat",
                    value: escape(value),
                    expected: "a non-negative integer",
                },
                "value {value:?}"
            );
        }
    }

    #[test]
    fn a_repeated_option_is_rejected() {
        assert_eq!(
            error(&[b"--color=auto", b"--color=always"]),
            UsageError::DuplicateOption("color")
        );

        assert_eq!(
            error(&[b"--hex", b"x", b"--hex"]),
            UsageError::DuplicateOption("hex")
        );
    }

    #[test]
    fn dependent_options_require_their_company() {
        assert_eq!(
            error(&[b"--separator=,"]),
            UsageError::MissingDependency {
                option: "separator",
                requires: "repeat",
            }
        );
        assert_eq!(
            error(&[b"--truncate"]),
            UsageError::MissingDependency {
                option: "truncate",
                requires: "width",
            }
        );
        assert_eq!(
            request(&[b"--repeat=2", b"--separator=,"]).options,
            vec![ResolvedOption::Repeat(2), ResolvedOption::Separator(b","),]
        );
        assert_eq!(
            request(&[b"--width=3", b"--truncate"]).options,
            vec![ResolvedOption::Width(3), ResolvedOption::Truncate]
        );
    }

    #[test]
    fn the_first_dependent_option_is_reported() {
        assert_eq!(
            error(&[b"--truncate", b"--separator=,"]),
            UsageError::MissingDependency {
                option: "truncate",
                requires: "width",
            }
        );
    }

    #[test]
    fn help_and_version_are_sole_argument_only() {
        assert_eq!(parse_bytes(&[b"--help"]), Ok(Parsed::Help));
        assert_eq!(parse_bytes(&[b"--version"]), Ok(Parsed::Version));

        assert_eq!(
            error(&[b"--help", b"x"]),
            UsageError::UnknownOption("--help".to_string())
        );
        assert_eq!(
            error(&[b"x", b"--version"]),
            UsageError::UnknownOption("--version".to_string())
        );
    }

    #[test]
    fn text_values_keep_arbitrary_bytes() {
        let parsed = request(&[b"--repeat=1", b"--separator=\xFF\x00"]);
        assert_eq!(parsed.options[1], ResolvedOption::Separator(b"\xFF\x00"));
    }

    #[test]
    fn encoders_are_mutually_exclusive() {
        assert_eq!(
            error(&[b"--hex", b"--base64=standard"]),
            UsageError::ConflictingOptions {
                option: "base64",
                conflicts_with: "hex",
            }
        );

        assert_eq!(
            error(&[b"--json", b"--hex"]),
            UsageError::ConflictingOptions {
                option: "hex",
                conflicts_with: "json",
            }
        );
        assert_eq!(
            request(&[b"--url=full"]).options,
            vec![ResolvedOption::Url(UrlMode::Full)]
        );
    }

    #[test]
    fn align_requires_width() {
        assert_eq!(
            error(&[b"--align=right"]),
            UsageError::MissingDependency {
                option: "align",
                requires: "width",
            }
        );
        assert_eq!(
            request(&[b"--align=right", b"--width=5"]).options,
            vec![
                ResolvedOption::Align(Align::Right),
                ResolvedOption::Width(5)
            ]
        );
    }
}
