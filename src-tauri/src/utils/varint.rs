/// Encodes the provided integer as a VarInt.
pub fn encode(num: u64) -> Vec<u8> {
    let bytes = num.to_le_bytes();
    if num < 0xfd {
        return vec![bytes[0]];
    } else if num < 0x10000 {
        return vec![0xfd, bytes[0], bytes[1]];
    } else if num < 0x100000000 {
        return vec![0xfe, bytes[0], bytes[1], bytes[2], bytes[3]];
    } else {
        return vec![
            0xff, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ];
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::varint;

    #[test]
    fn test_one_byte() {
        assert_eq!(varint::encode(0), vec![0x00]);
        assert_eq!(varint::encode(23), vec![0x17]);
        assert_eq!(varint::encode(252), vec![0xfc]);
    }

    #[test]
    fn test_two_bytes() {
        assert_eq!(varint::encode(253), vec![0xfd, 0xfd, 0x00]);
        assert_eq!(varint::encode(65535), vec![0xfd, 0xff, 0xff]);
    }

    #[test]
    fn test_four_bytes() {
        assert_eq!(varint::encode(65536), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(
            varint::encode(4294967295),
            vec![0xfe, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_eight_bytes() {
        assert_eq!(
            varint::encode(4294967296),
            vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            varint::encode(18446744073709551615),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }
}
