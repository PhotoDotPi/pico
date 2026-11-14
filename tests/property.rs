#[path = "../src/testing.rs"]
mod testing;

use std::process::Command;

use testing::Rng;

const OUTPUT_BOUND: usize = 64 * 1024 * 1024 + 1;

const TOKENS: &[&str] = &[
    "--extended",
    "--hex",
    "--json",
    "--base64=urlsafe",
    "--url=full",
    "--case=upper",
    "--width=4",
    "--truncate",
    "--align=right",
    "--repeat=2",
    "--repeat=0",
    "--repeat=9999999",
    "--separator=, ",
    "--number-lines",
    "--color=always",
    "--color=never",
    "--color=auto",
    "--style=success",
    "--",
    "-",
    "--bogus",
    "--repeat=x",
];

#[test]
fn random_invocations_keep_the_observable_contract() {
    let mut rng = Rng::new(0x5EED);
    for iteration in 0..48 {
        let count = 1 + usize::try_from(rng.below(6)).unwrap_or(0);
        let mut args: Vec<String> = Vec::with_capacity(count);
        for _ in 0..count {
            if rng.below(2) == 0 {
                let operand = rng.printable(12);
                args.push(String::from_utf8(operand).expect("printable ASCII is UTF-8"));
            } else {
                args.push(rng.choice(TOKENS).to_string());
            }
        }

        let mut invocation = vec!["--extended".to_string()];
        invocation.extend(args.into_iter().filter(|arg| arg != "--extended"));

        let output = Command::new(env!("CARGO_BIN_EXE_pico"))
            .args(&invocation)
            .output()
            .expect("spawn pico");

        let code = output
            .status
            .code()
            .unwrap_or_else(|| panic!("iteration {iteration} killed by a signal: {invocation:?}"));
        assert!(
            code == 0 || code == 1,
            "iteration {iteration} exited {code}: {invocation:?}"
        );
        assert!(
            output.stdout.len() <= OUTPUT_BOUND,
            "iteration {iteration} exceeded the output bound: {invocation:?}"
        );

        if output.status.success() {
            assert_eq!(output.stderr, b"", "iteration {iteration}: {invocation:?}");
        } else {
            assert_eq!(
                output.stdout, b"",
                "iteration {iteration} wrote partial output: {invocation:?}"
            );
            assert!(
                output.stderr.starts_with(b"pico: ") && output.stderr.ends_with(b"\n"),
                "iteration {iteration} diagnostic: {:?}",
                output.stderr
            );

            let body = &output.stderr[..output.stderr.len() - 1];
            assert!(
                !body.contains(&b'\n'),
                "iteration {iteration} diagnostic: {:?}",
                output.stderr
            );
        }
    }
}
