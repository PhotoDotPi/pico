use std::process::Command;

fn pico() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pico"))
}

fn assert_usage_error(args: &[&str], expected_stderr: &[u8]) {
    let out = pico().args(args).output().expect("run pico");
    assert_eq!(out.status.code(), Some(1), "args {args:?}");
    assert_eq!(out.stdout, b"", "no partial output for {args:?}");
    assert_eq!(out.stderr, expected_stderr, "args {args:?}");
}

fn assert_output(args: &[&str], expected_stdout: &[u8]) {
    let out = pico().args(args).output().expect("run pico");
    assert!(out.status.success(), "args {args:?}: {out:?}");
    assert_eq!(out.stdout, expected_stdout, "args {args:?}");
    assert_eq!(out.stderr, b"", "args {args:?}");
}

#[test]
fn activates_as_a_plain_echo() {
    let out = pico()
        .args(["--extended", "hello", "world"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hello world\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn no_operands_writes_only_a_newline() {
    let out = pico().arg("--extended").output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn does_not_interpret_backslash_escapes() {
    let out = pico()
        .args(["--extended", r"a\tb"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"a\\tb\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn preserves_empty_and_unicode_operands() {
    let args = ["--extended", "", "café", "😀", "e\u{0301}", ""];

    let out = pico().args(args).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, " café 😀 e\u{0301} \n".as_bytes());
    assert_eq!(out.stderr, b"");
}

#[test]
fn pico_compat_disables_extended_mode() {
    let out = pico()
        .env("PICO_COMPAT", "1")
        .args(["--extended", "--color=always"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--extended --color=always\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn activation_requires_the_first_position() {
    let out = pico().args(["x", "--extended"]).output().expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"x --extended\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn options_may_follow_operands() {
    assert_output(&["--extended", "hello", "--case=upper"], b"HELLO\n");
}

#[test]
fn the_end_of_options_marker_makes_everything_data() {
    let out = pico()
        .args(["--extended", "--", "--color=always", "-n", "--"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"--color=always -n --\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn a_lone_dash_is_an_operand() {
    let out = pico()
        .args(["--extended", "-", "x"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"- x\n");
    assert_eq!(out.stderr, b"");
}

#[test]
fn help_is_recognised_as_the_sole_argument() {
    let out = pico()
        .args(["--extended", "--help"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert!(out.stdout.starts_with(b"Usage: pico --extended"));
    assert_eq!(out.stderr, b"");
}

#[test]
fn version_is_recognised_as_the_sole_argument() {
    let out = pico()
        .args(["--extended", "--version"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    assert!(
        text.starts_with("pico "),
        "unexpected version text: {text:?}"
    );
    assert_eq!(out.stderr, b"");
}

#[test]
fn help_is_not_recognised_next_to_other_arguments() {
    assert_usage_error(
        &["--extended", "--help", "x"],
        b"pico: unknown option '--help'\n",
    );
}

#[test]
fn unknown_options_are_rejected() {
    assert_usage_error(
        &["--extended", "--bogus"],
        b"pico: unknown option '--bogus'\n",
    );

    assert_usage_error(&["--extended", "-n", "hi"], b"pico: unknown option '-n'\n");
}

#[test]
fn missing_values_are_rejected() {
    assert_usage_error(
        &["--extended", "--color"],
        b"pico: option '--color' requires a value\n",
    );
}

#[test]
fn invalid_enum_values_are_rejected() {
    assert_usage_error(
        &["--extended", "--color=red"],
        b"pico: invalid value 'red' for option '--color' (expected one of: auto, always, never)\n",
    );
    assert_usage_error(
        &["--extended", "--align=center"],
        b"pico: invalid value 'center' for option '--align' (expected one of: left, right, centre)\n",
    );
}

#[test]
fn invalid_counts_are_rejected() {
    assert_usage_error(
        &["--extended", "--repeat=-1"],
        b"pico: invalid value '-1' for option '--repeat' (expected a non-negative integer)\n",
    );
    assert_usage_error(
        &["--extended", "--width=18446744073709551616"],
        b"pico: invalid value '18446744073709551616' for option '--width' (expected a non-negative integer)\n",
    );
}

#[test]
fn flags_reject_values() {
    assert_usage_error(
        &["--extended", "--hex=1"],
        b"pico: option '--hex' does not take a value\n",
    );
}

#[test]
fn repeated_options_are_rejected() {
    assert_usage_error(
        &["--extended", "--color=auto", "--color=always"],
        b"pico: option '--color' specified more than once\n",
    );
}

#[test]
fn conflicting_options_are_rejected() {
    assert_usage_error(
        &["--extended", "--separator=,"],
        b"pico: option '--separator' requires option '--repeat'\n",
    );
    assert_usage_error(
        &["--extended", "--truncate"],
        b"pico: option '--truncate' requires option '--width'\n",
    );
}

#[test]
fn repetition_copies_the_text_the_requested_number_of_times() {
    assert_output(&["--extended", "--repeat=3", "ab"], b"ababab\n");

    assert_output(&["--extended", "--repeat=0", "hi"], b"\n");

    assert_output(
        &["--extended", "--repeat=1", "--separator=-", "hi"],
        b"hi\n",
    );

    assert_output(
        &["--extended", "--repeat=100", "x"],
        [b"x".repeat(100).as_slice(), b"\n"].concat().as_slice(),
    );
}

#[test]
fn the_separator_joins_repetitions_and_is_not_encoded() {
    assert_output(
        &["--extended", "--repeat=3", "--separator=-", "ab"],
        b"ab-ab-ab\n",
    );

    assert_output(
        &["--extended", "--hex", "--repeat=2", "--separator=:", "hi"],
        b"6869:6869\n",
    );

    assert_output(
        &[
            "--extended",
            "--case=upper",
            "--repeat=2",
            "--separator=-",
            "hi",
        ],
        b"HI-HI\n",
    );

    assert_output(
        &["--extended", "--repeat=2", "--separator=|", "a", "b"],
        b"a b|a b\n",
    );
}

#[test]
fn repetition_of_an_empty_body_repeats_only_the_separator() {
    assert_output(&["--extended", "--repeat=4", "--separator=-", ""], b"---\n");

    assert_output(&["--extended", "--repeat=18446744073709551615", ""], b"\n");
}

#[test]
fn the_trailing_newline_is_never_repeated() {
    assert_output(&["--extended", "--repeat=2", "a\n"], b"a\na\n\n");
}

#[test]
fn repetition_constrains_the_whole_line_under_width() {
    assert_output(
        &[
            "--extended",
            "--repeat=3",
            "--separator=-",
            "--width=4",
            "ab",
        ],
        b"ab-ab-ab\n",
    );
    assert_output(
        &[
            "--extended",
            "--repeat=2",
            "--separator=-",
            "--width=12",
            "ab",
        ],
        b"ab-ab       \n",
    );

    assert_output(
        &[
            "--extended",
            "--repeat=3",
            "--separator=-",
            "--width=5",
            "--truncate",
            "ab",
        ],
        b"ab-ab\n",
    );
}

#[test]
fn an_oversized_repetition_is_refused_before_output() {
    assert_usage_error(
        &["--extended", "--repeat=40000000", "hi"],
        b"pico: refusing to write more than 67108864 bytes (resource limit)\n",
    );

    assert_usage_error(
        &["--extended", "--repeat=18446744073709551615", "hi"],
        b"pico: refusing to write more than 67108864 bytes (resource limit)\n",
    );

    assert_usage_error(
        &["--extended", "--repeat=20000000", "--separator=abcd", "hi"],
        b"pico: refusing to write more than 67108864 bytes (resource limit)\n",
    );
}

#[test]
fn lines_are_numbered_one_based_with_a_tab() {
    assert_output(&["--extended", "--number-lines", "a\nb"], b"1\ta\n2\tb\n");

    assert_output(&["--extended", "--number-lines", "hi"], b"1\thi\n");

    let twelve = (1..=12)
        .map(|number| format!("{number}\tline"))
        .collect::<Vec<_>>()
        .join("\n");
    let body = (1..=12).map(|_| "line").collect::<Vec<_>>().join("\n");
    let expected = format!("{twelve}\n");
    assert_output(
        &["--extended", "--number-lines", &body],
        expected.as_bytes(),
    );
}

#[test]
fn a_trailing_newline_terminates_the_last_line_rather_than_adding_one() {
    assert_output(&["--extended", "--number-lines", "a\n"], b"1\ta\n");

    assert_output(&["--extended", "--number-lines", "\n"], b"1\t\n");

    assert_output(
        &["--extended", "--number-lines", "a\n\nb"],
        b"1\ta\n2\t\n3\tb\n",
    );
}

#[test]
fn an_empty_body_has_no_lines_just_the_mandatory_newline() {
    assert_output(&["--extended", "--number-lines", ""], b"\n");
}

#[test]
fn carriage_return_is_content_not_a_line_ending() {
    assert_output(
        &["--extended", "--number-lines", "a\r\nb"],
        b"1\ta\r\n2\tb\n",
    );
}

#[test]
fn numbering_prefixes_are_not_counted_in_the_width() {
    assert_output(
        &["--extended", "--number-lines", "--width=4", "hi"],
        b"1\thi  \n",
    );
    assert_output(
        &[
            "--extended",
            "--number-lines",
            "--width=4",
            "--align=right",
            "hi",
        ],
        b"1\t  hi\n",
    );
}

#[test]
fn numbering_numbers_the_repeated_body() {
    assert_output(
        &[
            "--extended",
            "--repeat=2",
            "--separator=\n",
            "--number-lines",
            "a",
        ],
        b"1\ta\n2\ta\n",
    );
}

#[test]
fn colour_always_wraps_the_whole_line_in_sgr_sequences() {
    assert_output(
        &["--extended", "--color=always", "hi"],
        b"\x1b[36mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=success", "hi"],
        b"\x1b[32mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=error", "hi"],
        b"\x1b[31mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=warning", "hi"],
        b"\x1b[33mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=information", "hi"],
        b"\x1b[36mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=muted", "hi"],
        b"\x1b[2mhi\x1b[0m\n",
    );
    assert_output(
        &["--extended", "--color=always", "--style=emphasis", "hi"],
        b"\x1b[1mhi\x1b[0m\n",
    );
}

#[test]
fn colour_never_emits_no_escapes_even_with_a_style() {
    assert_output(
        &["--extended", "--color=never", "--style=error", "hi"],
        b"hi\n",
    );
}

#[test]
fn auto_colour_never_fires_on_redirected_output() {
    assert_output(
        &["--extended", "--color=auto", "--style=error", "hi"],
        b"hi\n",
    );
    assert_output(&["--extended", "--style=error", "hi"], b"hi\n");
}

#[test]
fn no_color_suppresses_auto_colour_but_not_an_explicit_choice() {
    let out = pico()
        .env("NO_COLOR", "1")
        .args(["--extended", "--color=auto", "--style=error", "hi"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"hi\n");

    let out = pico()
        .env("NO_COLOR", "1")
        .args(["--extended", "--color=always", "--style=error", "hi"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\x1b[31mhi\x1b[0m\n");

    let out = pico()
        .env("NO_COLOR", "")
        .args(["--extended", "--color=always", "hi"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\x1b[36mhi\x1b[0m\n");
}

#[test]
fn plain_output_contains_no_escape_bytes() {
    let out = pico()
        .args(["--extended", "--hex", "--case=upper", "hi"])
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"4849\n");
    assert!(!out.stdout.contains(&0x1B), "no ESC in plain output");
}

#[test]
fn escape_sequences_are_never_encoded_and_never_counted_in_the_width() {
    assert_output(
        &["--extended", "--color=always", "--hex", "hi"],
        b"\x1b[36m6869\x1b[0m\n",
    );

    assert_output(
        &["--extended", "--color=always", "--width=5", "hi"],
        b"\x1b[36mhi   \x1b[0m\n",
    );

    assert_output(
        &[
            "--extended",
            "--color=always",
            "--style=success",
            "--number-lines",
            "a\nb",
        ],
        b"\x1b[32m1\ta\n2\tb\x1b[0m\n",
    );

    assert_output(
        &[
            "--extended",
            "--color=always",
            "--repeat=2",
            "--separator=-",
            "hi",
        ],
        b"\x1b[36mhi-hi\x1b[0m\n",
    );
}

#[test]
fn an_invalid_style_value_is_rejected() {
    assert_usage_error(
        &["--extended", "--style=blink"],
        b"pico: invalid value 'blink' for option '--style' (expected one of: success, error, warning, information, muted, emphasis)\n",
    );
}

#[test]
fn hex_encodes_the_joined_operands() {
    assert_output(&["--extended", "--hex", "A"], b"41\n");

    assert_output(&["--extended", "--hex", "a", "b"], b"612062\n");

    assert_output(&["--extended", "--hex", "é"], b"c3a9\n");

    assert_output(&["--extended", "--hex", ""], b"\n");
}

#[test]
fn base64_variants_match_rfc_4648() {
    assert_output(
        &["--extended", "--base64=standard", "foobar"],
        b"Zm9vYmFy\n",
    );
    assert_output(&["--extended", "--base64=standard", "f"], b"Zg==\n");
    assert_output(&["--extended", "--base64=standard", "ab"], b"YWI=\n");
    assert_output(&["--extended", "--base64=standard-nopad", "f"], b"Zg\n");
    assert_output(&["--extended", "--base64=standard-nopad", "ab"], b"YWI\n");
    assert_output(&["--extended", "--base64=urlsafe", "foobar"], b"Zm9vYmFy\n");
    assert_output(&["--extended", "--base64=urlsafe-nopad", "f"], b"Zg\n");
}

#[test]
fn json_wraps_the_text_in_a_string() {
    assert_output(&["--extended", "--json", "foo"], b"\"foo\"\n");
    assert_output(&["--extended", "--json", "a\"b"], b"\"a\\\"b\"\n");
    assert_output(&["--extended", "--json", "a\\b"], b"\"a\\\\b\"\n");
    assert_output(&["--extended", "--json", ""], b"\"\"\n");

    assert_output(&["--extended", "--json", "a\tb"], b"\"a\\tb\"\n");

    assert_output(&["--extended", "--json", "café"], "\"café\"\n".as_bytes());
}

#[test]
fn url_encoding_distinguishes_component_and_full() {
    assert_output(&["--extended", "--url=component", "a b"], b"a%20b\n");
    assert_output(&["--extended", "--url=component", "a/b"], b"a%2Fb\n");
    assert_output(
        &["--extended", "--url=component", "日本"],
        b"%E6%97%A5%E6%9C%AC\n",
    );
    assert_output(
        &["--extended", "--url=full", "https://x/a b?q=1"],
        b"https://x/a%20b?q=1\n",
    );
}

#[test]
fn case_modes_convert_text() {
    assert_output(&["--extended", "--case=upper", "café"], "CAFÉ\n".as_bytes());
    assert_output(&["--extended", "--case=lower", "CAFÉ"], "café\n".as_bytes());

    assert_output(
        &["--extended", "--case=ascii-upper", "café"],
        "CAFé\n".as_bytes(),
    );
    assert_output(
        &["--extended", "--case=ascii-lower", "CAFé"],
        "café\n".as_bytes(),
    );

    assert_output(
        &["--extended", "--case=upper", "Straße"],
        "STRASSE\n".as_bytes(),
    );
}

#[test]
fn width_pads_and_aligns() {
    assert_output(&["--extended", "--width=5", "hi"], b"hi   \n");
    assert_output(
        &["--extended", "--width=5", "--align=right", "hi"],
        b"   hi\n",
    );
    assert_output(
        &["--extended", "--width=5", "--align=centre", "hi"],
        b" hi  \n",
    );

    assert_output(&["--extended", "--width=2", "hi"], b"hi\n");

    assert_output(&["--extended", "--width=3", ""], b"   \n");
}

#[test]
fn truncation_cuts_to_width_without_splitting_scalars() {
    assert_output(
        &["--extended", "--width=3", "--truncate", "abcdef"],
        b"abc\n",
    );
    assert_output(&["--extended", "--width=0", "--truncate", "hello"], b"\n");

    assert_output(
        &["--extended", "--width=2", "--truncate", "日本語"],
        "日本\n".as_bytes(),
    );
}

#[test]
fn width_counts_scalars_not_display_columns() {
    assert_output(&["--extended", "--width=4", "é"], "é   \n".as_bytes());

    assert_output(
        &["--extended", "--width=5", "e\u{0301}"],
        "e\u{0301}   \n".as_bytes(),
    );

    assert_output(
        &["--extended", "--width=3", "--align=right", "日"],
        "  日\n".as_bytes(),
    );

    assert_output(
        &["--extended", "--width=1", "--truncate", "👨‍👩‍👧"],
        "👨\n".as_bytes(),
    );
}

#[test]
fn implemented_features_combine_in_a_fixed_order() {
    assert_output(
        &[
            "--extended",
            "--case=upper",
            "--hex",
            "--width=6",
            "--align=right",
            "hi",
        ],
        b"  4849\n",
    );
}

#[test]
fn conflicting_encoders_are_rejected_before_output() {
    assert_usage_error(
        &["--extended", "--hex", "--base64=standard", "hi"],
        b"pico: option '--base64' cannot be combined with '--hex'\n",
    );
}

#[test]
fn align_without_width_is_rejected() {
    assert_usage_error(
        &["--extended", "--align=right", "hi"],
        b"pico: option '--align' requires option '--width'\n",
    );
}

#[test]
fn oversized_output_is_refused_before_output() {
    assert_usage_error(
        &["--extended", "--width=100000000", "hi"],
        b"pico: refusing to write more than 67108864 bytes (resource limit)\n",
    );
}

#[cfg(unix)]
#[test]
fn invalid_utf8_operands_are_handled_byte_for_byte() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    let raw = OsStr::from_bytes(b"\xFF\xFE");
    let out = pico()
        .args([OsStr::new("--extended"), OsStr::new("--hex")])
        .arg(raw)
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"fffe\n");

    let out = pico()
        .args([OsStr::new("--extended"), OsStr::new("--width=3")])
        .arg(raw)
        .output()
        .expect("run pico");
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\xFF\xFE \n");
}
