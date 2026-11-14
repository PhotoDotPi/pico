pub(crate) fn split_lines(body: &[u8]) -> Vec<&[u8]> {
    if body.is_empty() {
        return Vec::new();
    }

    let mut lines: Vec<&[u8]> = body.split(|&byte| byte == b'\n').collect();
    if body.ends_with(b"\n") {
        lines.pop();
    }
    lines
}

pub(crate) fn prefix_bytes(line_count: u64) -> u64 {
    let mut total = line_count;
    if line_count == 0 {
        return 0;
    }

    let mut digits: u64 = 1;
    let mut start: u64 = 1;
    while start <= line_count {
        let end = start.saturating_mul(10).saturating_sub(1).min(line_count);
        let numbers = end - start + 1;
        total = total.saturating_add(numbers.saturating_mul(digits));
        digits += 1;
        match start.checked_mul(10) {
            Some(next) => start = next,
            None => break,
        }
    }

    total
}

pub(crate) fn apply(body: &[u8], left: u64, right: u64) -> Vec<u8> {
    let lines = split_lines(body);
    let capacity = body
        .len()
        .saturating_add(usize::try_from(prefix_bytes(lines.len() as u64)).unwrap_or(0))
        .saturating_add(usize::try_from(left.saturating_add(right)).unwrap_or(0));

    let mut output = Vec::with_capacity(capacity);
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push(b'\n');
        }
        write_number(&mut output, index as u64 + 1);
        output.push(b'\t');
        if index == 0 {
            push_spaces(&mut output, left);
        }
        output.extend_from_slice(line);
    }
    if lines.is_empty() {
        push_spaces(&mut output, left);
    }
    push_spaces(&mut output, right);
    output
}

fn push_spaces(output: &mut Vec<u8>, mut count: u64) {
    const SPACES: [u8; 64] = [b' '; 64];
    while count >= 64 {
        output.extend_from_slice(&SPACES);
        count -= 64;
    }
    if count > 0 {
        output.extend_from_slice(&SPACES[..usize::try_from(count).unwrap_or(0)]);
    }
}

fn write_number(output: &mut Vec<u8>, value: u64) {
    let mut buffer = [0u8; 20];
    let mut index = buffer.len();
    let mut remaining = value;
    loop {
        index -= 1;
        buffer[index] = b'0' + u8::try_from(remaining % 10).unwrap_or(0);
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    output.extend_from_slice(&buffer[index..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digits(mut n: u64) -> u64 {
        let mut count = 1;
        while n >= 10 {
            n /= 10;
            count += 1;
        }
        count
    }

    fn number(body: &[u8]) -> Vec<u8> {
        apply(body, 0, 0)
    }

    #[test]
    fn empty_body_has_no_lines() {
        assert_eq!(split_lines(b""), Vec::<&[u8]>::new());
        assert_eq!(number(b""), b"");
        assert_eq!(prefix_bytes(0), 0);
    }

    #[test]
    fn a_trailing_newline_terminates_rather_than_adds() {
        assert_eq!(split_lines(b"a"), vec![&b"a"[..]]);
        assert_eq!(split_lines(b"a\n"), vec![&b"a"[..]]);
        assert_eq!(number(b"a"), b"1\ta");
        assert_eq!(number(b"a\n"), b"1\ta");
    }

    #[test]
    fn a_lone_newline_is_one_empty_line() {
        assert_eq!(split_lines(b"\n"), vec![&b""[..]]);
        assert_eq!(number(b"\n"), b"1\t");
    }

    #[test]
    fn embedded_newlines_each_start_a_numbered_line() {
        assert_eq!(number(b"a\nb"), b"1\ta\n2\tb");
        assert_eq!(number(b"a\nb\n"), b"1\ta\n2\tb");
        assert_eq!(number(b"a\n\nb"), b"1\ta\n2\t\n3\tb");
        assert_eq!(number(b"\n\n"), b"1\t\n2\t");
    }

    #[test]
    fn carriage_return_is_content_not_a_line_ending() {
        assert_eq!(split_lines(b"a\r\nb"), vec![&b"a\r"[..], &b"b"[..]]);
        assert_eq!(number(b"a\r\nb"), b"1\ta\r\n2\tb");
    }

    #[test]
    fn numbers_are_one_based_and_decimal() {
        let body = b"a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk";
        let text = String::from_utf8(number(body)).expect("UTF-8");
        assert!(text.starts_with("1\ta\n2\tb\n"));
        assert!(text.ends_with("10\tj\n11\tk"));
    }

    #[test]
    fn padding_is_placed_inside_the_numbered_lines() {
        assert_eq!(apply(b"hi", 0, 2), b"1\thi  ");
        assert_eq!(apply(b"hi", 2, 0), b"1\t  hi");
        assert_eq!(apply(b"a\nb", 0, 2), b"1\ta\n2\tb  ");
        assert_eq!(apply(b"a\nb", 2, 0), b"1\t  a\n2\tb");

        assert_eq!(apply(b"a\n", 0, 2), b"1\ta  ");

        assert_eq!(apply(b"", 2, 1), b"   ");
    }

    #[test]
    fn prefix_bytes_matches_a_direct_summation() {
        fn reference(line_count: u64) -> u64 {
            (1..=line_count).map(|n| digits(n) + 1).sum()
        }
        for count in [0u64, 1, 2, 9, 10, 11, 99, 100, 101, 1000, 123_456] {
            assert_eq!(prefix_bytes(count), reference(count), "count {count}");
        }
    }

    #[test]
    fn digits_counts_decimal_digits() {
        assert_eq!(digits(0), 1);
        assert_eq!(digits(9), 1);
        assert_eq!(digits(10), 2);
        assert_eq!(digits(99), 2);
        assert_eq!(digits(100), 3);
        assert_eq!(digits(u64::MAX), 20);
    }

    #[test]
    fn apply_length_is_joined_lines_plus_prefixes_and_padding() {
        for body in [&b""[..], &b"a"[..], &b"a\nb\nc\n"[..], &b"\n\n\n"[..]] {
            let lines = split_lines(body);
            let joined: u64 = lines.iter().map(|line| line.len() as u64).sum::<u64>()
                + lines.len().saturating_sub(1) as u64;
            let expected = joined + prefix_bytes(lines.len() as u64) + 5;
            assert_eq!(apply(body, 2, 3).len() as u64, expected);
        }
    }
}
