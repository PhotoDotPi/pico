use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};

#[derive(Debug)]
struct Case {
    name: &'static str,

    args: Vec<OsString>,

    envs: Vec<(OsString, OsString)>,

    compare_stdout: bool,

    compare_stderr: bool,
}

impl Case {
    fn new(name: &'static str, args: &[&str]) -> Self {
        Self {
            name,
            args: args.iter().map(OsString::from).collect(),
            envs: Vec::new(),
            compare_stdout: true,
            compare_stderr: true,
        }
    }

    #[must_use]
    fn with_env(mut self, key: &str, value: &str) -> Self {
        self.envs.push((OsString::from(key), OsString::from(value)));
        self
    }

    #[must_use]
    fn without_stdout(mut self) -> Self {
        self.compare_stdout = false;
        self
    }

    fn from_owned(name: &'static str, args: Vec<OsString>) -> Self {
        Self {
            name,
            args,
            envs: Vec::new(),
            compare_stdout: true,
            compare_stderr: true,
        }
    }
}

fn reference_executable() -> Result<Option<PathBuf>, String> {
    match env::var_os("PICO_REFERENCE_ECHO") {
        None => Ok(None),
        Some(value) if value.is_empty() => Ok(None),
        Some(value) => {
            let path = PathBuf::from(&value);
            if path.is_file() {
                Ok(Some(path))
            } else {
                Err(format!(
                    "PICO_REFERENCE_ECHO={value:?} does not name an existing file"
                ))
            }
        }
    }
}

fn invoke(program: &Path, case: &Case) -> Output {
    let mut command = Command::new(program);
    command.args(&case.args);

    command.env_remove("POSIXLY_CORRECT");
    command.env_remove("PICO_COMPAT");
    command.env("LC_ALL", "C");
    for (key, value) in &case.envs {
        command.env(key, value);
    }
    command
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()))
}

fn describe_status(status: ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }

    match status.code() {
        Some(code) => format!("exit status {code}"),
        None => "terminated without an exit status".to_string(),
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    if bytes.is_empty() {
        return "<empty>".to_string();
    }

    let mut rendered = String::with_capacity(bytes.len() * 3);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            rendered.push(' ');
        }

        let _ = write!(rendered, "{byte:02X}");
    }
    rendered
}

fn compare_case(reference: &Path, pico: &Path, case: &Case) -> Result<(), String> {
    let expected = invoke(reference, case);
    let actual = invoke(pico, case);

    let mut differences = Vec::new();

    let expected_status = describe_status(expected.status);
    let actual_status = describe_status(actual.status);
    if expected_status != actual_status {
        differences.push(format!(
            "exit status: reference={expected_status} pico={actual_status}"
        ));
    }

    if case.compare_stdout && expected.stdout != actual.stdout {
        differences.push(format!(
            "stdout:\n    reference = {}\n    pico      = {}",
            hex(&expected.stdout),
            hex(&actual.stdout)
        ));
    }

    if case.compare_stderr && expected.stderr != actual.stderr {
        differences.push(format!(
            "stderr:\n    reference = {}\n    pico      = {}",
            hex(&expected.stderr),
            hex(&actual.stderr)
        ));
    }

    if differences.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "case {:?} args={:?}\n  {}",
            case.name,
            case.args,
            differences.join("\n  ")
        ))
    }
}

fn reference_version(reference: &Path) -> String {
    match Command::new(reference).arg("--version").output() {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("<no version line>")
            .to_string(),
        _ => "<unable to determine reference version>".to_string(),
    }
}

fn session_02_cases() -> Vec<Case> {
    vec![
        Case::new("COMP-001 no arguments", &[]),
        Case::new("COMP-002 single operand", &["hello"]),
        Case::new("COMP-002 two operands", &["hello", "world"]),
        Case::new("COMP-002 three operands", &["a", "b", "c"]),
        Case::new("COMP-002 spaces inside an operand", &["a b", "c"]),
        Case::new("COMP-003 single empty", &[""]),
        Case::new("COMP-003 empty-x-empty", &["", "x", ""]),
        Case::new("COMP-003 two empties", &["", ""]),
        Case::new("COMP-004 -n alone", &["-n"]),
        Case::new("COMP-004 -n with an operand", &["-n", "hi"]),
        Case::new("COMP-004 -n with two operands", &["-n", "a", "b"]),
        Case::new("COMP-004 -n with an empty operand", &["-n", ""]),
        Case::new("COMP-005 -e one operand", &["-e", "hi"]),
        Case::new("COMP-005 -e two operands", &["-e", "a", "b"]),
        Case::new("COMP-006 -E one operand", &["-E", "hi"]),
        Case::new("COMP-006 -E after -e", &["-e", "-E", "hi"]),
        Case::new("COMP-007 -ne", &["-ne", "hi"]),
        Case::new("COMP-007 -en", &["-en", "hi"]),
        Case::new("COMP-007 -nE", &["-nE", "hi"]),
        Case::new("COMP-007 -eE", &["-eE", "hi"]),
        Case::new("COMP-007 -Ee", &["-Ee", "hi"]),
        Case::new("COMP-007 -nn", &["-nn", "hi"]),
        Case::new("COMP-008 -x", &["-x", "hi"]),
        Case::new("COMP-008 -nex", &["-nex", "hi"]),
        Case::new("COMP-008 -abc", &["-abc"]),
        Case::new("COMP-008 lone dash", &["-", "hi"]),
        Case::new("COMP-008 dash before an option-like operand", &["-", "-n"]),
        Case::new("COMP-009 double dash alone", &["--"]),
        Case::new("COMP-009 double dash then -n", &["--", "-n"]),
        Case::new("COMP-009 double dash then operands", &["--", "x", "y"]),
        Case::new("COMP-010 sole --help", &["--help"]).without_stdout(),
        Case::new("COMP-010 sole --version", &["--version"]).without_stdout(),
        Case::new("COMP-010 --help with an operand", &["--help", "x"]),
        Case::new("COMP-010 --version with an operand", &["--version", "x"]),
        Case::new("COMP-010 operand then --help", &["x", "--help"]),
        Case::new("COMP-010 --help then --version", &["--help", "--version"]),
        Case::new("COMP-011 -n then -- then x", &["-n", "--", "x"]),
        Case::new("COMP-011 -n then operand then -e", &["-n", "x", "-e"]),
        Case::new("COMP-011 operand then -n", &["x", "-n"]),
        Case::new("COMP-011 -e then operand then -n", &["-e", "x", "-n"]),
        Case::new("COMP-012 unknown option exits zero", &["-x"]),
        Case::new("COMP-014 plain output", &["plain"]),
        Case::new(
            "ENV-004 PICO_COMPAT=1 makes --extended data",
            &["--extended"],
        )
        .with_env("PICO_COMPAT", "1"),
    ]
}

fn session_03_cases() -> Vec<Case> {
    vec![
        Case::new("ESC-001 double backslash", &["-e", r"a\\b"]),
        Case::new("ESC-002 alert", &["-e", r"\a"]),
        Case::new("ESC-002 backspace", &["-e", r"\b"]),
        Case::new("ESC-002 form feed", &["-e", r"\f"]),
        Case::new("ESC-002 newline", &["-e", r"\n"]),
        Case::new("ESC-002 carriage return", &["-e", r"\r"]),
        Case::new("ESC-002 tab", &["-e", r"\t"]),
        Case::new("ESC-002 vertical tab", &["-e", r"\v"]),
        Case::new(
            "ESC-002 all singles in one operand",
            &["-e", r"\a\b\f\n\r\t\v"],
        ),
        Case::new("ESC-003 escape byte", &["-e", r"\e"]),
        Case::new("ESC-004 c then text", &["-e", r"a\cb"]),
        Case::new("ESC-004 lone c", &["-e", r"\c"]),
        Case::new("ESC-004 c with -n", &["-ne", r"a\cb"]),
        Case::new("ESC-005 bare NUL", &["-e", r"\0"]),
        Case::new("ESC-005 octal 0101", &["-e", r"\0101"]),
        Case::new("ESC-005 three digits 0123", &["-e", r"\0123"]),
        Case::new("ESC-005 extra digit is data", &["-e", r"\01011"]),
        Case::new("ESC-005 non-octal after NUL", &["-e", r"\08"]),
        Case::new("ESC-006 single digit", &["-e", r"\1"]),
        Case::new("ESC-006 two digits", &["-e", r"\12"]),
        Case::new("ESC-006 octal 101", &["-e", r"\101"]),
        Case::new("ESC-006 three digits then data", &["-e", r"\1011"]),
        Case::new("ESC-006 8 is not octal", &["-e", r"\8"]),
        Case::new("ESC-007 hex 41", &["-e", r"\x41"]),
        Case::new("ESC-007 one hex digit", &["-e", r"\x4"]),
        Case::new("ESC-007 lowercase ff", &["-e", r"\xff"]),
        Case::new("ESC-007 one digit then data", &["-e", r"\x0g"]),
        Case::new("ESC-007 two digits then data", &["-e", r"\x414"]),
        Case::new("ESC-008 q", &["-e", r"\q"]),
        Case::new("ESC-008 in context", &["-e", r"a\qz"]),
        Case::new("ESC-008 backslash-space", &["-e", r"\ "]),
        Case::new("ESC-009 trailing backslash", &["-e", "a\\"]),
        Case::new("ESC-009 lone backslash", &["-e", "\\"]),
        Case::new("ESC-010 x at end", &["-e", r"\x"]),
        Case::new("ESC-010 x non-hex", &["-e", r"\xg"]),
        Case::new("ESC-010 xZZ", &["-e", r"\xZZ"]),
        Case::new("ESC-011 777", &["-e", r"\777"]),
        Case::new("ESC-011 400", &["-e", r"\400"]),
        Case::new("ESC-011 0777", &["-e", r"\0777"]),
        Case::new("ESC-012 c stops later operand", &["-e", r"a\c", "b"]),
        Case::new("ESC-013 c in second operand", &["-e", "a", r"b\cc", "d"]),
        Case::new("ESC-013 empty operand then c", &["-e", "", r"\c"]),
        Case::new("ESC combined sequence", &["-e", r"one\ttwo\nthree\x21\041"]),
        Case::new("COMP-006 escapes off by default", &[r"a\tb"]),
        Case::new("COMP-006 escapes off with -E", &["-E", r"a\tb"]),
        Case::new("COMP-005 -e with no escapes", &["-e", "plain"]),
        Case::new("COMP-015 embedded NUL", &["-e", r"a\000b"]),
        Case::new("ENV-001 escapes enabled", &[r"a\tb"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-001 -e treated as data", &["-e", r"a\tb"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-001 -x treated as data", &["-x", "hi"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-001 --help treated as data", &["--help"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-001 empty value enables escapes", &[r"a\nb"])
            .with_env("POSIXLY_CORRECT", ""),
        Case::new("ENV-002 leading -n honoured", &["-n", "hi"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-002 leading -n alone", &["-n"]).with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-002 leading -n then -e", &["-n", "-e", r"a\tb"])
            .with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-002 leading -n then -E", &["-n", "-E", r"a\tb"])
            .with_env("POSIXLY_CORRECT", "1"),
        Case::new("ENV-003 c still exits zero", &[r"a\cb"]).with_env("POSIXLY_CORRECT", "1"),
    ]
}

#[cfg(unix)]
fn session_03_unix_cases() -> Vec<Case> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    vec![
        Case::from_owned(
            "COMP-013 non-UTF-8 operand passthrough",
            vec![OsString::from(OsStr::from_bytes(&[0xFF, 0xFE]))],
        ),
        Case::from_owned(
            "COMP-013 non-UTF-8 operand with escape",
            vec![
                OsString::from("-e"),
                OsString::from(OsStr::from_bytes(&[0xFF, b'\\', b't', 0xFE])),
            ],
        ),
        Case::from_owned(
            "COMP-015 non-UTF-8 operand with NUL escape",
            vec![
                OsString::from("-e"),
                OsString::from(OsStr::from_bytes(&[0xFF, b'\\', b'0', 0xFE])),
            ],
        ),
    ]
}

#[cfg(not(unix))]
fn session_03_unix_cases() -> Vec<Case> {
    Vec::new()
}

fn session_04_cases() -> Vec<Case> {
    vec![
        Case::from_owned(
            "COMP-016 long argument",
            vec![OsString::from("a".repeat(10_000))],
        ),
        Case::from_owned(
            "COMP-016 many arguments",
            (0..4_000)
                .map(|index| OsString::from(index.to_string()))
                .collect(),
        ),
        Case::from_owned(
            "COMP-016 long escape-heavy operand",
            vec![OsString::from("-e"), OsString::from(r"\t".repeat(3_000))],
        ),
        Case::new(
            "COMP-017 unicode operands",
            &["café", "日本語", "😀🎉", "e\u{0301}", "Ω≈ç√∫"],
        ),
        Case::new("COMP-017 unicode with escapes", &["-e", "héllo\\twörld 😀"]),
        Case::new("COMP-017 unicode and empties", &["", "café", "", "😀", ""]),
    ]
}

#[cfg(target_os = "linux")]
fn session_04_boundary_cases() -> Vec<Case> {
    vec![
        Case::from_owned(
            "COMP-016 near-limit argument",
            vec![OsString::from("z".repeat(120_000))],
        ),
        Case::from_owned(
            "COMP-016 very many arguments",
            (0..20_000)
                .map(|index| OsString::from(index.to_string()))
                .collect(),
        ),
    ]
}

#[cfg(not(target_os = "linux"))]
fn session_04_boundary_cases() -> Vec<Case> {
    Vec::new()
}

#[test]
fn compatibility_matches_reference() {
    let reference = match reference_executable() {
        Ok(Some(path)) => path,
        Ok(None) => {
            eprintln!(
                "PICO_REFERENCE_ECHO is not set: differential compatibility tests SKIPPED. \
                 Configure a reference to enable them."
            );
            return;
        }
        Err(message) => panic!("differential tests are misconfigured: {message}"),
    };

    let pico = PathBuf::from(env!("CARGO_BIN_EXE_pico"));
    eprintln!(
        "differential reference: {} ({})",
        reference.display(),
        reference_version(&reference)
    );

    let cases = session_02_cases()
        .into_iter()
        .chain(session_03_cases())
        .chain(session_03_unix_cases())
        .chain(session_04_cases())
        .chain(session_04_boundary_cases())
        .collect::<Vec<_>>();
    eprintln!("running {} differential case(s)", cases.len());
    let mut failures = Vec::new();
    for case in &cases {
        if let Err(problem) = compare_case(&reference, &pico, case) {
            failures.push(problem);
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} differential case(s) failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n\n")
    );
}
