//! Implements the Base58 Encoding Scheme as specified
//! in https://tools.ietf.org/id/draft-msporny-base58-02.html.
use lazy_static::lazy_static;
use num_bigint::BigUint;
use std::collections::HashMap;

static BASE58_ALPHABET: [char; 58] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z',
];

lazy_static! {
    static ref BASE58_DECIMAL: HashMap<char, u16> = {
        let mut m = HashMap::new();
        m.insert('1', 0);
        m.insert('2', 1);
        m.insert('3', 2);
        m.insert('4', 3);
        m.insert('5', 4);
        m.insert('6', 5);
        m.insert('7', 6);
        m.insert('8', 7);
        m.insert('9', 8);
        m.insert('A', 9);
        m.insert('B', 10);
        m.insert('C', 11);
        m.insert('D', 12);
        m.insert('E', 13);
        m.insert('F', 14);
        m.insert('G', 15);
        m.insert('H', 16);
        m.insert('J', 17);
        m.insert('K', 18);
        m.insert('L', 19);
        m.insert('M', 20);
        m.insert('N', 21);
        m.insert('P', 22);
        m.insert('Q', 23);
        m.insert('R', 24);
        m.insert('S', 25);
        m.insert('T', 26);
        m.insert('U', 27);
        m.insert('V', 28);
        m.insert('W', 29);
        m.insert('X', 30);
        m.insert('Y', 31);
        m.insert('Z', 32);
        m.insert('a', 33);
        m.insert('b', 34);
        m.insert('c', 35);
        m.insert('d', 36);
        m.insert('e', 37);
        m.insert('f', 38);
        m.insert('g', 39);
        m.insert('h', 40);
        m.insert('i', 41);
        m.insert('j', 42);
        m.insert('k', 43);
        m.insert('m', 44);
        m.insert('n', 45);
        m.insert('o', 46);
        m.insert('p', 47);
        m.insert('q', 48);
        m.insert('r', 49);
        m.insert('s', 50);
        m.insert('t', 51);
        m.insert('u', 52);
        m.insert('v', 53);
        m.insert('w', 54);
        m.insert('x', 55);
        m.insert('y', 56);
        m.insert('z', 57);
        m
    };
}

/// Encodes the bytes to Base58.
pub fn encode(bytes: &[u8]) -> String {
    let zero: BigUint = BigUint::from(0u8);
    let two: BigUint = BigUint::from(2u8);

    let mut count = 0;
    for b in bytes {
        if *b == 0 {
            count += 1;
        } else {
            break;
        }
    }

    let mut decimal_value = bytes
        .iter()
        .rev()
        .enumerate()
        .fold(zero.clone(), |sum, (i, b)| {
            sum + *b * two.pow((i as u32) * 8)
        });

    let mut encoding = String::new();
    while decimal_value > zero {
        let remainder = decimal_value.clone() % 58u8;
        let index: usize = remainder.try_into().unwrap();
        encoding.insert(0, BASE58_ALPHABET[index]);
        decimal_value /= 58u8;
    }
    (0..count).for_each(|_| encoding.insert(0, '1'));
    return encoding;
}

/// Decodes the string from Base58 to bytes.
pub fn decode(s: &str) -> Option<Vec<u8>> {
    if s.chars().any(|c| !BASE58_DECIMAL.contains_key(&c)) {
        return None;
    }
    let orig_len = s.len();
    let s = s.trim_start_matches('1');
    let len = s.len();
    let zero: BigUint = BigUint::from(0u8);
    let fifty_eight: BigUint = BigUint::from(58u8);
    let two_fifty_six: BigUint = BigUint::from(256u16);
    let mut decimal_value = s
        .chars()
        .rev()
        .enumerate()
        .fold(zero.clone(), |sum, (i, c)| {
            sum + BASE58_DECIMAL.get(&c).unwrap() * fifty_eight.pow(i as u32)
        });
    let mut bytes: Vec<u8> = Vec::new();
    while decimal_value.clone() > zero {
        let val = decimal_value.clone() % two_fifty_six.clone();
        decimal_value = decimal_value.clone() / two_fifty_six.clone();
        bytes.push(val.try_into().unwrap());
    }
    bytes.reverse();
    (0..(orig_len - len)).for_each(|_| bytes.insert(0, 0x00));
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use crate::utils::base58;

    #[test]
    fn test_base58_encode() {
        let s = b"asdfg4d04kblm58slq";
        let target_b58 = String::from("53SBFPHJ2fYLpcrVQBRy8st9v");
        let b58 = base58::encode(s);
        assert_eq!(b58, target_b58);
    }

    #[test]
    fn test_base58_decode() {
        let b58 = "hQgQJ9mLxjdhSrmqbiFy1HXkezgWP9bQ99";
        let target_s = String::from("dlbaSl391032flml20s0x1lsd");
        let decoded = base58::decode(b58).unwrap();
        let s = String::from_utf8(decoded).unwrap();
        assert_eq!(s, target_s);
    }
}
