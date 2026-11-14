#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};

const LARGE_OPERAND_LEN: usize = 100_000;

#[cfg(target_os = "linux")]
fn pico() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pico"))
}

fn pico_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pico"))
}

fn reference_or_skip() -> Option<PathBuf> {
    match std::env::var_os("PICO_REFERENCE_ECHO") {
        None => None,
        Some(value) if value.is_empty() => None,
        Some(value) => {
            let path = PathBuf::from(&value);
            assert!(
                path.is_file(),
                "PICO_REFERENCE_ECHO={value:?} does not name an existing file"
            );
            Some(path)
        }
    }
}

fn signal(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt as _;
    status.signal()
}

fn run_without_a_reader(program: &Path, operand: &str) -> Output {
    let mut child = Command::new(program)
        .arg(operand)
        .env_remove("POSIXLY_CORRECT")
        .env_remove("PICO_COMPAT")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("failed to spawn {}: {error}", program.display()));

    drop(child.stdout.take());

    child
        .wait_with_output()
        .unwrap_or_else(|error| panic!("failed to wait for {}: {error}", program.display()))
}

#[test]
fn pico_broken_pipe_exits_141_without_a_diagnostic() {
    let operand = "x".repeat(LARGE_OPERAND_LEN);
    let out = run_without_a_reader(&pico_path(), &operand);

    assert_eq!(
        out.status.code(),
        Some(141),
        "expected a normal exit with status 141, got {:?}",
        out.status
    );
    assert_eq!(
        out.stderr, b"",
        "no diagnostic is expected on a broken pipe"
    );
}

#[test]
fn broken_pipe_matches_the_reference_shell_visible_status() {
    let Some(reference) = reference_or_skip() else {
        eprintln!("PICO_REFERENCE_ECHO is not set: broken-pipe reference test SKIPPED");
        return;
    };

    let operand = "x".repeat(LARGE_OPERAND_LEN);
    let expected = run_without_a_reader(&reference, &operand);
    let actual = run_without_a_reader(&pico_path(), &operand);

    assert_eq!(
        signal(expected.status),
        Some(13),
        "reference should be terminated by SIGPIPE, got {:?}",
        expected.status
    );
    assert_eq!(expected.stderr, b"");

    assert_eq!(actual.status.code(), Some(141));
    assert_eq!(actual.stderr, b"");
}

#[cfg(target_os = "linux")]
#[test]
fn pico_write_error_exits_1_with_a_diagnostic() {
    let out = pico()
        .arg("hello")
        .env_remove("POSIXLY_CORRECT")
        .env_remove("PICO_COMPAT")
        .stdout(open_dev_full())
        .output()
        .expect("run pico");

    assert_eq!(out.status.code(), Some(1), "status {:?}", out.status);
    assert_eq!(out.stderr, b"pico: write error\n");
}

#[cfg(target_os = "linux")]
#[test]
fn write_error_matches_the_reference_exit_status() {
    let Some(reference) = reference_or_skip() else {
        eprintln!("PICO_REFERENCE_ECHO is not set: write-error reference test SKIPPED");
        return;
    };

    let expected = Command::new(&reference)
        .arg("hello")
        .env_remove("POSIXLY_CORRECT")
        .env_remove("PICO_COMPAT")
        .stdout(open_dev_full())
        .output()
        .expect("run reference");

    assert_eq!(expected.status.code(), Some(1));
    assert!(
        expected.stderr.starts_with(b"echo: write error"),
        "unexpected reference diagnostic: {:?}",
        String::from_utf8_lossy(&expected.stderr)
    );
}

#[cfg(target_os = "linux")]
fn open_dev_full() -> std::fs::File {
    std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full should be writable")
}
