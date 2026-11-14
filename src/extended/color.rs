use crate::extended::parser::{ColorWhen, Style};

pub(crate) const RESET: &[u8] = b"\x1b[0m";

pub(crate) const DEFAULT_STYLE: Style = Style::Information;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColorEnvironment {
    pub(crate) no_color: bool,

    pub(crate) stdout_is_terminal: bool,
}

pub(crate) fn prefix(style: Style) -> &'static [u8] {
    match style {
        Style::Success => b"\x1b[32m",
        Style::Error => b"\x1b[31m",
        Style::Warning => b"\x1b[33m",
        Style::Information => b"\x1b[36m",

        Style::Muted => b"\x1b[2m",
        Style::Emphasis => b"\x1b[1m",
    }
}

pub(crate) fn enabled(when: ColorWhen, environment: ColorEnvironment) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => environment.stdout_is_terminal && !environment.no_color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(no_color: bool, terminal: bool) -> ColorEnvironment {
        ColorEnvironment {
            no_color,
            stdout_is_terminal: terminal,
        }
    }

    #[test]
    fn always_enables_regardless_of_environment() {
        for no_color in [false, true] {
            for terminal in [false, true] {
                assert!(enabled(ColorWhen::Always, env(no_color, terminal)));
            }
        }
    }

    #[test]
    fn never_disables_regardless_of_environment() {
        for no_color in [false, true] {
            for terminal in [false, true] {
                assert!(!enabled(ColorWhen::Never, env(no_color, terminal)));
            }
        }
    }

    #[test]
    fn auto_needs_a_terminal_and_no_no_color() {
        assert!(enabled(ColorWhen::Auto, env(false, true)));

        assert!(!enabled(ColorWhen::Auto, env(false, false)));

        assert!(!enabled(ColorWhen::Auto, env(true, true)));
        assert!(!enabled(ColorWhen::Auto, env(true, false)));
    }

    #[test]
    fn every_style_has_a_distinct_sgr_prefix_and_a_shared_reset() {
        let styles = [
            Style::Success,
            Style::Error,
            Style::Warning,
            Style::Information,
            Style::Muted,
            Style::Emphasis,
        ];
        for style in styles {
            let bytes = prefix(style);
            assert!(bytes.starts_with(b"\x1b["));
            assert!(bytes.ends_with(b"m"));
        }
        assert_eq!(prefix(Style::Success), b"\x1b[32m");
        assert_eq!(prefix(Style::Error), b"\x1b[31m");
        assert_eq!(prefix(Style::Warning), b"\x1b[33m");
        assert_eq!(prefix(Style::Information), b"\x1b[36m");
        assert_eq!(prefix(Style::Muted), b"\x1b[2m");
        assert_eq!(prefix(Style::Emphasis), b"\x1b[1m");
        assert_eq!(RESET, b"\x1b[0m");
    }
}
