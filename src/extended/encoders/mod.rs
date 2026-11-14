pub(crate) mod base64;
pub(crate) mod hex;
pub(crate) mod json;
pub(crate) mod url;

use crate::extended::parser::{Base64, UrlMode};

pub(super) const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

pub(super) const UPPER_HEX: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoder {
    Hex,

    Base64(Base64),

    Json,

    Url(UrlMode),
}

pub(crate) fn encode(input: &[u8], encoder: Encoder) -> Vec<u8> {
    match encoder {
        Encoder::Hex => hex::encode(input),
        Encoder::Base64(variant) => base64::encode(input, variant),
        Encoder::Json => json::encode(input),
        Encoder::Url(mode) => url::encode(input, mode),
    }
}
