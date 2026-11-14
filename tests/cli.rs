use std::process::Command;

fn pico() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pico"))
}

#[test]
fn no_arguments_prints_just_a_newline() {
    let out = pico().output().expect("run pico");
    assert!(out.status.success(), "exit status: {:?}", out.status);
    assert_eq!(out.stdout, b"\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn joins_arguments_with_a_single_space() {
    let out = pico().args(["hello", "world"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hello world\n");
}

#[test]
fn preserves_empty_arguments() {
    let out = pico().args(["", "x", ""]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b" x \n");
}

#[test]
fn does_not_interpret_backslash_escapes_by_default() {
    let out = pico().arg(r"a\tb").output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\\tb\n");
}

#[test]
fn minus_n_suppresses_the_trailing_newline() {
    let out = pico().args(["-n", "hi"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hi");
    assert_eq!(out.stderr, b"");
}

#[test]
fn lone_minus_n_writes_nothing() {
    let out = pico().arg("-n").output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"");
    assert_eq!(out.stderr, b"");
}

#[test]
fn clustered_options_are_applied_left_to_right() {
    let cases: &[(&str, &[u8])] = &[
        ("-ne", b"hi"),
        ("-en", b"hi"),
        ("-nE", b"hi"),
        ("-nn", b"hi"),
        ("-eE", b"hi\n"),
        ("-Ee", b"hi\n"),
    ];

    for (cluster, expected) in cases {
        let out = pico().args([*cluster, "hi"]).output().expect("run pico");
        assert!(out.status.success(), "cluster {cluster}");
        assert_eq!(out.stdout, *expected, "cluster {cluster}");
        assert_eq!(out.stderr, b"", "cluster {cluster}");
    }
}

#[test]
fn unknown_options_are_data_and_exit_zero() {
    for args in [vec!["-x", "hi"], vec!["-nex", "hi"], vec!["-abc"]] {
        let out = pico().args(&args).output().expect("run pico");
        assert!(out.status.success(), "{args:?}");
        let expected = format!("{}\n", args.join(" "));
        assert_eq!(out.stdout, expected.as_bytes(), "{args:?}");
        assert_eq!(out.stderr, b"", "{args:?}");
    }
}

#[test]
fn double_dash_and_lone_dash_are_data() {
    let out = pico().args(["--", "-n"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"-- -n\n");

    let out = pico().args(["-", "-n"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"- -n\n");
}

#[test]
fn option_parsing_stops_at_the_first_operand() {
    let out = pico().args(["-n", "x", "-e"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"x -e");

    let out = pico().args(["-n", "--", "x"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"-- x");
}

#[test]
fn help_and_version_are_recognised_only_as_the_sole_argument() {
    let out = pico().arg("--help").output().expect("run pico");
    assert!(out.status.success());
    assert!(out.stdout.starts_with(b"Usage: pico"));

    let out = pico().args(["--help", "x"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--help x\n");

    let out = pico().arg("--version").output().expect("run pico");
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    assert!(
        text.starts_with("pico "),
        "unexpected version text: {text:?}"
    );

    let out = pico().args(["--version", "x"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--version x\n");
}

#[test]
fn minus_e_interprets_backslash_escapes() {
    let out = pico().args(["-e", r"a\tb"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\tb\n");
    assert_eq!(out.stderr, b"");

    let out = pico().arg(r"a\tb").output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\\tb\n");

    let out = pico().args(["-E", r"a\tb"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\\tb\n");
}

#[test]
fn minus_e_escape_forms_produce_the_expected_bytes() {
    let cases: &[(&str, &[u8])] = &[
        (r"\a", b"\x07\n"),
        (r"\b", b"\x08\n"),
        (r"\f", b"\x0C\n"),
        (r"\n", b"\x0A\n"),
        (r"\r", b"\x0D\n"),
        (r"\t", b"\x09\n"),
        (r"\v", b"\x0B\n"),
        (r"\e", b"\x1B\n"),
        (r"\\", b"\\\n"),
        (r"\101", b"A\n"),
        (r"\0", b"\x00\n"),
        (r"\0101", b"A\n"),
        (r"\x41", b"A\n"),
        (r"\x4", b"\x04\n"),
        (r"\q", b"\\q\n"),
        (r"\x", b"\\x\n"),
        (r"a\", b"a\\\n"),
    ];

    for (operand, expected) in cases {
        let out = pico().args(["-e", operand]).output().expect("run pico");
        assert!(out.status.success(), "operand {operand:?}");
        assert_eq!(out.stdout, *expected, "operand {operand:?}");
        assert_eq!(out.stderr, b"", "operand {operand:?}");
    }
}

#[test]
fn backslash_c_stops_output_and_suppresses_the_newline() {
    let out = pico().args(["-e", r"a\cb"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a");
    assert_eq!(out.stderr, b"");

    let out = pico().args(["-e", r"a\c", "b"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a");
}

#[test]
fn minus_e_preserves_embedded_nul_bytes() {
    let out = pico().args(["-e", r"a\000b"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\x00b\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn posixly_correct_enables_escapes_and_demotes_options() {
    let out = pico()
        .env("POSIXLY_CORRECT", "1")
        .arg(r"a\tb")
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\tb\n");

    let out = pico()
        .env("POSIXLY_CORRECT", "1")
        .args(["-e", r"a\tb"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"-e a\tb\n");

    let out = pico()
        .env("POSIXLY_CORRECT", "1")
        .args(["-n", "hi"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hi");

    let out = pico()
        .env("POSIXLY_CORRECT", "")
        .arg(r"a\nb")
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\nb\n");

    assert_eq!(out.stderr, b"");
}

#[test]
fn posixly_correct_makes_help_literal() {
    let out = pico()
        .env("POSIXLY_CORRECT", "1")
        .arg("--help")
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--help\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn pico_compat_env_restores_gnu_behaviour_for_extended() {
    let out = pico()
        .env("PICO_COMPAT", "1")
        .arg("--extended")
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--extended\n");
    assert_eq!(out.stderr, b"");
}

#[cfg(unix)]
#[test]
fn preserves_non_utf8_arguments_on_unix() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    let raw = OsStr::from_bytes(b"\xFF\xFE");
    let out = pico().arg(raw).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\xFF\xFE\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn unicode_arguments_pass_through_unchanged() {
    let args = ["café", "日本語", "😀🎉", "e\u{0301}", "Ω≈ç√∫"];

    let out = pico().args(args).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, expected_join(&args));
    assert_eq!(out.stderr, b"");
}

#[test]
fn unicode_survives_escape_interpretation() {
    let out = pico()
        .args(["-e", "héllo\\twörld 😀"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, "héllo\twörld 😀\n".as_bytes());
    assert_eq!(out.stderr, b"");
}

#[test]
fn empty_arguments_mixed_with_unicode() {
    let args = ["", "café", "", "😀", ""];
    let out = pico().args(args).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, expected_join(&args));
    assert_eq!(out.stderr, b"");
}

#[test]
fn a_long_argument_is_written_verbatim() {
    let operand = "a".repeat(10_000);

    let out = pico().arg(&operand).output().expect("run pico");
    assert!(out.status.success());

    let mut expected = operand.into_bytes();
    expected.push(b'\n');
    assert_eq!(out.stdout, expected);
    assert_eq!(out.stderr, b"");
}

#[test]
fn many_arguments_are_joined_in_order() {
    let args: Vec<String> = (0..4_000).map(|index| index.to_string()).collect();

    let out = pico().args(&args).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, expected_join(&args));
    assert_eq!(out.stderr, b"");
}

#[test]
fn a_long_escape_heavy_operand_is_interpreted() {
    let operand = r"\t".repeat(3_000);

    let out = pico().args(["-e", &operand]).output().expect("run pico");
    assert!(out.status.success());

    let mut expected = vec![b'\t'; 3_000];
    expected.push(b'\n');
    assert_eq!(out.stdout, expected);
    assert_eq!(out.stderr, b"");
}

fn expected_join(args: &[impl AsRef<str>]) -> Vec<u8> {
    let mut expected = Vec::new();
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            expected.push(b' ');
        }
        expected.extend_from_slice(arg.as_ref().as_bytes());
    }
    expected.push(b'\n');
    expected
}
