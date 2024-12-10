pub fn one_at_a_time(input_bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    for b in input_bytes {
        hash = hash.overflowing_add(*b as u32).0;
        hash = hash.overflowing_add(hash << 10).0;
        hash ^= hash >> 6;
    }
    hash = hash.overflowing_add(hash << 3).0;
    hash ^= hash >> 11;
    hash = hash.overflowing_add(hash << 15).0;
    hash as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn one_at_a_time() {
        assert_eq!(super::one_at_a_time("a".as_bytes()), 0xca2e9442);
        assert_eq!(
            super::one_at_a_time("The quick brown fox jumps over the lazy dog".as_bytes()),
            0x519e91f5
        );
    }
}
