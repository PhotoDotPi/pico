use crate::extended::parser::UsageError;

pub(crate) const MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) fn too_large() -> UsageError {
    UsageError::OutputTooLarge {
        limit: MAX_OUTPUT_BYTES,
    }
}

pub(crate) fn check_total(parts: impl IntoIterator<Item = u64>) -> Result<(), UsageError> {
    let mut total: u64 = 0;
    for part in parts {
        total = total.checked_add(part).ok_or(UsageError::OutputTooLarge {
            limit: MAX_OUTPUT_BYTES,
        })?;
    }

    if total > MAX_OUTPUT_BYTES {
        Err(UsageError::OutputTooLarge {
            limit: MAX_OUTPUT_BYTES,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(body_len: usize, padding: u64) -> Result<(), UsageError> {
        check_total([u64::try_from(body_len).unwrap_or(u64::MAX), padding, 1])
    }

    #[test]
    fn small_output_is_allowed() {
        assert_eq!(check(0, 0), Ok(()));
        assert_eq!(check(1024, 0), Ok(()));
        assert_eq!(check(1024, 1024), Ok(()));
    }

    #[test]
    fn the_limit_is_inclusive() {
        let limit = MAX_OUTPUT_BYTES;
        assert_eq!(check(usize::try_from(limit - 1).unwrap(), 0), Ok(()));
        assert!(check(usize::try_from(limit).unwrap(), 0).is_err());
    }

    #[test]
    fn padding_alone_can_exceed_the_limit() {
        assert!(check(0, MAX_OUTPUT_BYTES).is_err());
        assert!(check(0, u64::MAX).is_err());
    }

    #[test]
    fn overflow_is_a_clean_error() {
        assert!(check(usize::MAX, u64::MAX).is_err());
    }

    #[test]
    fn check_total_sums_parts_with_checked_arithmetic() {
        assert_eq!(check_total([1, 2, 3]), Ok(()));
        assert_eq!(check_total(std::iter::empty()), Ok(()));

        assert!(check_total([MAX_OUTPUT_BYTES, 1]).is_err());
        assert!(check_total([MAX_OUTPUT_BYTES - 1, 2]).is_err());
        assert_eq!(check_total([MAX_OUTPUT_BYTES, 0]), Ok(()));

        assert!(check_total([u64::MAX, 1]).is_err());
        assert!(check_total([u64::MAX, u64::MAX]).is_err());
    }
}
