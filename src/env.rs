use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Env {
    pub(crate) pico_compat: bool,

    pub(crate) posixly_correct: bool,

    pub(crate) no_color: bool,
}

impl Env {
    pub(crate) fn from_process() -> Self {
        Self::from_lookup(|name| std::env::var_os(name))
    }

    fn from_lookup(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Self {
        Self {
            pico_compat: lookup("PICO_COMPAT")
                .as_deref()
                .is_some_and(|value| value.as_encoded_bytes() == b"1"),

            posixly_correct: lookup("POSIXLY_CORRECT").is_some(),

            no_color: lookup("NO_COLOR")
                .as_deref()
                .is_some_and(|value| !value.as_encoded_bytes().is_empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_from(vars: &[(&str, &str)]) -> Env {
        let map: HashMap<String, OsString> = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), OsString::from(value)))
            .collect();
        Env::from_lookup(|key| map.get(key).cloned())
    }

    #[test]
    fn defaults_when_nothing_is_set() {
        let env = env_from(&[]);
        assert!(!env.pico_compat);
        assert!(!env.posixly_correct);
        assert!(!env.no_color);
    }

    #[test]
    fn no_color_counts_only_when_present_and_not_empty() {
        assert!(env_from(&[("NO_COLOR", "1")]).no_color);
        assert!(env_from(&[("NO_COLOR", "anything")]).no_color);
        assert!(env_from(&[("NO_COLOR", " ")]).no_color);

        assert!(!env_from(&[("NO_COLOR", "")]).no_color);
        assert!(!env_from(&[]).no_color);
    }

    #[test]
    fn pico_compat_requires_the_exact_value_one() {
        assert!(env_from(&[("PICO_COMPAT", "1")]).pico_compat);
        assert!(!env_from(&[("PICO_COMPAT", "")]).pico_compat);
        assert!(!env_from(&[("PICO_COMPAT", "0")]).pico_compat);
        assert!(!env_from(&[("PICO_COMPAT", "true")]).pico_compat);
        assert!(!env_from(&[("PICO_COMPAT", " 1")]).pico_compat);
    }

    #[test]
    fn posixly_correct_is_true_for_empty_and_non_empty_values() {
        assert!(env_from(&[("POSIXLY_CORRECT", "1")]).posixly_correct);
        assert!(env_from(&[("POSIXLY_CORRECT", "")]).posixly_correct);
        assert!(env_from(&[("POSIXLY_CORRECT", "anything")]).posixly_correct);
        assert!(!env_from(&[]).posixly_correct);
    }

    #[test]
    fn the_two_variables_are_independent() {
        let env = env_from(&[("PICO_COMPAT", "1"), ("POSIXLY_CORRECT", "1")]);
        assert!(env.pico_compat);
        assert!(env.posixly_correct);

        let env = env_from(&[("PICO_COMPAT", "1")]);
        assert!(env.pico_compat);
        assert!(!env.posixly_correct);
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_values_are_handled_without_panicking() {
        use std::os::unix::ffi::OsStringExt as _;

        let mut map: HashMap<String, OsString> = HashMap::new();
        map.insert(
            "PICO_COMPAT".to_string(),
            OsString::from_vec(vec![0xFF, 0xFE]),
        );
        map.insert(
            "POSIXLY_CORRECT".to_string(),
            OsString::from_vec(vec![0xFF, 0xFE]),
        );
        map.insert("NO_COLOR".to_string(), OsString::from_vec(vec![0xFF, 0xFE]));

        let env = Env::from_lookup(|key| map.get(key).cloned());
        assert!(!env.pico_compat);
        assert!(env.posixly_correct);
        assert!(env.no_color);
    }

    #[cfg(windows)]
    #[test]
    fn non_unicode_values_are_handled_without_panicking() {
        use std::os::windows::ffi::OsStringExt as _;

        let mut map: HashMap<String, OsString> = HashMap::new();
        map.insert("PICO_COMPAT".to_string(), OsString::from_wide(&[0xD800]));
        map.insert(
            "POSIXLY_CORRECT".to_string(),
            OsString::from_wide(&[0xD800]),
        );

        let env = Env::from_lookup(|key| map.get(key).cloned());
        assert!(!env.pico_compat);
        assert!(env.posixly_correct);
    }
}
