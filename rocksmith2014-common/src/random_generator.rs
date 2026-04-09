/// Random number generation utilities.
pub mod random {
    use std::sync::Mutex;

    static SEED: Mutex<u64> = Mutex::new(12345);

    fn next_u64() -> u64 {
        let mut s = SEED.lock().unwrap();
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    /// Returns a non-negative random integer.
    pub fn next() -> i32 {
        (next_u64() >> 33) as i32
    }

    /// Returns a random integer in `[min, max)`.
    pub fn next_in_range(min: i32, max: i32) -> i32 {
        let range = (max - min) as u64;
        min + (next_u64() % range) as i32
    }

    /// Returns a random lowercase alphabet character.
    pub fn next_alphabet() -> char {
        char::from_u32(b'a' as u32 + (next_u64() % 26) as u32).unwrap_or('a')
    }
}
