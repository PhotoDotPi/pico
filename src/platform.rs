use std::borrow::Cow;
use std::ffi::{OsStr, OsString};

pub(crate) fn args() -> Vec<OsString> {
    std::env::args_os().skip(1).collect()
}

pub(crate) fn arg_bytes(args: &[OsString]) -> Vec<Cow<'_, [u8]>> {
    args.iter().map(|arg| bytes_of(arg.as_os_str())).collect()
}

#[cfg(unix)]
pub(crate) fn bytes_of(arg: &OsStr) -> Cow<'_, [u8]> {
    use std::os::unix::ffi::OsStrExt as _;
    Cow::Borrowed(arg.as_bytes())
}

#[cfg(not(unix))]
pub(crate) fn bytes_of(arg: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(arg.as_encoded_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_ascii_round_trips() {
        let arg = OsString::from("hello");
        assert_eq!(bytes_of(arg.as_os_str()).as_ref(), b"hello");
    }

    #[test]
    fn empty_argument_is_an_empty_slice() {
        let arg = OsString::new();
        assert_eq!(bytes_of(arg.as_os_str()).as_ref(), b"");
    }

    #[test]
    fn arg_bytes_preserves_order_and_count() {
        let args = vec![OsString::from("a"), OsString::new(), OsString::from("c")];
        let bytes = arg_bytes(&args);
        assert_eq!(bytes.len(), 3);
        assert_eq!(bytes[0].as_ref(), b"a");
        assert_eq!(bytes[1].as_ref(), b"");
        assert_eq!(bytes[2].as_ref(), b"c");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_bytes_are_preserved_on_unix() {
        use std::os::unix::ffi::OsStrExt as _;
        let raw = [0xFFu8, 0xFE, 0x00, 0x41];
        let arg = OsStr::from_bytes(&raw);
        assert_eq!(bytes_of(arg).as_ref(), &raw);
    }

    #[test]
    fn unicode_text_is_preserved_as_exact_utf8() {
        let samples = ["café", "日本語", "e\u{0301}", "😀🎉", "Ω≈ç√∫"];

        for sample in samples {
            let arg = OsString::from(sample);
            assert_eq!(
                bytes_of(arg.as_os_str()).as_ref(),
                sample.as_bytes(),
                "sample {sample:?}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn unpaired_surrogates_are_preserved_without_lossy_replacement() {
        use std::os::windows::ffi::OsStringExt as _;

        let arg = OsString::from_wide(&[0xD800, 0x0041]);
        let bytes = bytes_of(arg.as_os_str());
        assert!(!bytes.is_empty());
        assert!(
            !bytes.windows(3).any(|window| window == [0xEF, 0xBF, 0xBD]),
            "a lossy U+FFFD replacement character must not appear: {bytes:02X?}"
        );
    }
}
