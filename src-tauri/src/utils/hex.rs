/// Converts a hex string to bytes.
pub fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    // taken from: https://users.rust-lang.org/t/hex-string-to-vec-u8/51903/3
    if s.len() % 2 == 0 {
        (0..s.len())
            .step_by(2)
            .map(|i| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
            .collect()
    } else {
        None
    }
}

/// Converts the bytes to a hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_str = String::new();
    bytes
        .iter()
        .for_each(|b| hex_str.push_str(&format!("{:02x}", b)));
    hex_str
}

#[cfg(test)]
mod tests {
    use crate::utils::hex;

    #[test]
    fn test_hex_to_bytes() {
        let hex = "a078b13a63f0e62d0500902224c91a534f8e81f4ce1a1a59a0656f72e7a7e8f4";
        let target_bytes = vec![
            0xa0, 0x78, 0xb1, 0x3a, 0x63, 0xf0, 0xe6, 0x2d, 0x05, 0x00, 0x90, 0x22, 0x24, 0xc9,
            0x1a, 0x53, 0x4f, 0x8e, 0x81, 0xf4, 0xce, 0x1a, 0x1a, 0x59, 0xa0, 0x65, 0x6f, 0x72,
            0xe7, 0xa7, 0xe8, 0xf4,
        ];
        let bytes = hex::hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, target_bytes);
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![
            0xa0, 0x78, 0xb1, 0x3a, 0x63, 0xf0, 0xe6, 0x2d, 0x05, 0x00, 0x90, 0x22, 0x24, 0xc9,
            0x1a, 0x53, 0x4f, 0x8e, 0x81, 0xf4, 0xce, 0x1a, 0x1a, 0x59, 0xa0, 0x65, 0x6f, 0x72,
            0xe7, 0xa7, 0xe8, 0xf4,
        ];
        let target_hex = "a078b13a63f0e62d0500902224c91a534f8e81f4ce1a1a59a0656f72e7a7e8f4";
        let hex = hex::bytes_to_hex(&bytes);
        assert_eq!(hex, target_hex);
    }
}
