//! Implements [BIP-39](https://en.bitcoin.it/wiki/BIP_0039).
//! This BIP consists of the generation of the mnemonic and converting
//! it into a binary seed.
use bitcoin_hashes::{sha256, Hash};
use bitvec::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::keys::pbkdf2;

/// Reads the word list and returns a vector of words.
fn load_word_list() -> Vec<&'static str> {
    const WORDS: &str = include_str!("word_list.txt");
    let mut word_list: Vec<&str> = WORDS.split("\n").collect();
    word_list.remove(word_list.len() - 1); // remove empty string
    word_list
}

/// Generates the mnemonic from an entropy and returns a vector of words.
/// If `entropy` is `None`, then random bytes are generated using a RNG.
pub fn generate_mnemonic(entropy_length: usize, entropy: Option<&[u8]>) -> Vec<&'static str> {
    assert!(
        [128, 160, 192, 224, 256].contains(&entropy_length),
        "entropy_length must be either 128, 160, 192, 224, or 256"
    );
    assert_eq!(
        entropy_length,
        entropy.map_or(entropy_length, |v| 8 * v.len())
    );

    let word_list = load_word_list();

    let random_bytes = match entropy {
        Some(entropy) => entropy.to_vec(),
        None => {
            let mut rng = ChaCha20Rng::from_entropy();
            let mut random_bytes = vec![0; entropy_length / 8];
            rng.fill_bytes(&mut random_bytes);
            random_bytes
        }
    };

    let hash = sha256::Hash::hash(&random_bytes);
    let hash_bytes: [u8; 32] = hash.into_inner();
    let hash_bits = hash_bytes.view_bits::<Msb0>();
    let (checksum_bits, _) = hash_bits.split_at(entropy_length / 32);
    let mut random_bits = random_bytes.view_bits::<Msb0>().to_bitvec();
    random_bits.extend(checksum_bits);
    random_bits
        .chunks(11)
        .map(|section| {
            let index: usize = section.load_be();
            word_list[index].clone()
        })
        .collect()
}

/// Generates the binary seed from the mnemonic.
pub fn generate_seed(mnemonic: Vec<&str>, passphrase: &str) -> [u8; 64] {
    let mut seed = [0u8; 64];
    pbkdf2::pbkdf2(mnemonic.into_iter(), passphrase.as_bytes(), 2048, &mut seed);
    seed
}

#[cfg(test)]
mod tests {
    use super::{generate_mnemonic, generate_seed, load_word_list};

    fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
        // taken from https://users.rust-lang.org/t/hex-string-to-vec-u8/51903/3
        if s.len() % 2 == 0 {
            (0..s.len())
                .step_by(2)
                .map(|i| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
                .collect()
        } else {
            None
        }
    }

    #[test]
    fn word_list_not_empty() {
        let word_list = load_word_list();
        assert!(word_list.len() > 0);
    }

    #[test]
    fn check_word_list() {
        let word_list = load_word_list();
        assert_eq!(word_list[0], "abandon");
        assert_eq!(word_list[word_list.len() - 1], "zoo");
    }

    #[test]
    fn test_generate_mnemonic_128() {
        let bytes = hex_to_bytes("0c1e24e5917779d297e14d45f14e1a1a").unwrap();
        let mnemonic = generate_mnemonic(8 * bytes.len(), Some(&bytes));
        let target_mnemonic = vec![
            "army", "van", "defense", "carry", "jealous", "true", "garbage", "claim", "echo",
            "media", "make", "crunch",
        ];
        assert_eq!(mnemonic, target_mnemonic);
    }

    #[test]
    fn test_generate_mnemonic_256() {
        let bytes =
            hex_to_bytes("2041546864449caff939d32d574753fe684d3c947c3346713dd8423e74abcf8c")
                .unwrap();
        let mnemonic = generate_mnemonic(8 * bytes.len(), Some(&bytes));
        let target_mnemonic = vec![
            "cake", "apple", "borrow", "silk", "endorse", "fitness", "top", "denial", "coil",
            "riot", "stay", "wolf", "luggage", "oxygen", "faint", "major", "edit", "measure",
            "invite", "love", "trap", "field", "dilemma", "oblige",
        ];
        assert_eq!(mnemonic, target_mnemonic);
    }

    #[test]
    fn test_generate_seed() {
        let mnemonic = vec![
            "army", "van", "defense", "carry", "jealous", "true", "garbage", "claim", "echo",
            "media", "make", "crunch",
        ];
        let target_seed = hex_to_bytes(
            "5b56c417303faa3fcba7e57400e120a0ca83ec5a4fc9ffba757fbe63fbd77a89a1a3be4c67196f57c39a88b76373733891bfaba16ed27a813ceed498804c0570",
        )
        .unwrap();
        let seed = generate_seed(mnemonic, "");
        assert_eq!(seed.to_vec(), target_seed);
    }

    #[test]
    fn test_generate_seed_passphrase() {
        let mnemonic = vec![
            "army", "van", "defense", "carry", "jealous", "true", "garbage", "claim", "echo",
            "media", "make", "crunch",
        ];
        let target_seed = hex_to_bytes(
            "3b5df16df2157104cfdd22830162a5e170c0161653e3afe6c88defeefb0818c793dbb28ab3ab091897d0715861dc8a18358f80b79d49acf64142ae57037d1d54",
        )
        .unwrap();
        let seed = generate_seed(mnemonic, "SuperDuperSecret");
        assert_eq!(seed.to_vec(), target_seed);
    }
}
