pub(crate) struct Rng {
    state: u64,
}

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            0
        } else {
            self.next_u64() % bound
        }
    }

    pub(crate) fn bytes(&mut self, max_len: usize) -> Vec<u8> {
        let length = self.bounded(max_len);
        (0..length).map(|_| self.next_u8()).collect()
    }

    pub(crate) fn printable(&mut self, max_len: usize) -> Vec<u8> {
        let length = self.bounded(max_len);
        (0..length)
            .map(|_| u8::try_from(0x20 + self.below(0x5F)).unwrap_or(b' '))
            .collect()
    }

    pub(crate) fn text(&mut self, max_chars: usize) -> Vec<u8> {
        const POOL: &[char] = &[
            'a', 'b', 'z', 'A', 'Z', '0', '9', ' ', '\t', '\n', '\n', 'é', 'ß', '日', '😀',
            '\u{0301}', '~',
        ];
        let length = self.bounded(max_chars);
        let mut text = Vec::new();
        for _ in 0..length {
            let index = self.bounded(POOL.len());
            let mut buffer = [0u8; 4];
            text.extend_from_slice(POOL[index].encode_utf8(&mut buffer).as_bytes());
        }
        text
    }

    pub(crate) fn choice<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.bounded(items.len())]
    }

    fn bounded(&mut self, bound: usize) -> usize {
        usize::try_from(self.below(u64::try_from(bound).unwrap_or(u64::MAX))).unwrap_or(0)
    }

    fn next_u8(&mut self) -> u8 {
        u8::try_from(self.next_u64() % 256).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_reproduces_the_same_sequence() {
        let mut first = Rng::new(42);
        let mut second = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(first.next_u64(), second.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut first = Rng::new(1);
        let mut second = Rng::new(2);
        assert_ne!(first.next_u64(), second.next_u64());
    }

    #[test]
    fn a_zero_seed_is_remapped_to_a_non_zero_state() {
        let mut rng = Rng::new(0);
        let first = rng.next_u64();
        assert_ne!(first, 0);
        let mut again = Rng::new(0);
        assert_eq!(first, again.next_u64());
    }

    #[test]
    fn bounds_are_respected() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            assert!(rng.below(5) < 5);
            assert!(rng.bytes(8).len() < 8);
            assert!(rng.printable(8).len() < 8);
            assert!(rng.text(8).len() <= 32);
            assert!(
                std::str::from_utf8(&rng.text(8))
                    .expect("UTF-8")
                    .chars()
                    .count()
                    < 8
            );
        }
        assert_eq!(rng.below(0), 0);
    }

    #[test]
    fn text_is_always_valid_utf8_and_esc_free() {
        let mut rng = Rng::new(99);
        for _ in 0..500 {
            let text = rng.text(32);
            let decoded = std::str::from_utf8(&text).expect("valid UTF-8");
            assert!(!decoded.contains('\u{1B}'), "no ESC in generated text");
        }
    }

    #[test]
    fn printable_is_always_printable_ascii_and_esc_free() {
        let mut rng = Rng::new(1234);
        for _ in 0..500 {
            for byte in rng.printable(32) {
                assert!((0x20..0x7F).contains(&byte), "byte {byte:#04X}");
            }
        }
    }

    #[test]
    fn choice_returns_a_member_of_the_pool() {
        let mut rng = Rng::new(5);
        let pool = [10u32, 20, 30];
        for _ in 0..100 {
            assert!(pool.contains(&rng.choice(&pool)));
        }
    }
}
