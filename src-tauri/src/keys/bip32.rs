//! Implements [BIP-32](https://en.bitcoin.it/wiki/BIP_0032).
//! This BIP describes hierarchical deterministic wallets (or "HD Wallets").
use crate::keys::address::Address;
use crate::keys::error::{ChildKeyDeriveError, ImportKeyError};
use crate::utils::base58;
use bitcoin_hashes::{hmac, ripemd160, sha256, sha512, Hash, HashEngine};
use num_bigint::BigUint;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtendedPrivateKey {
    pub testnet: bool,
    pub depth: u8,
    pub fingerprint: [u8; 4],
    pub child_number: [u8; 4],
    pub chain_code: [u8; 32],
    pub key_data: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtendedPublicKey {
    pub testnet: bool,
    pub depth: u8,
    pub fingerprint: [u8; 4],
    pub child_number: [u8; 4],
    pub chain_code: [u8; 32],
    pub key_data: Vec<u8>, // must be 33 bytes
}

impl ExtendedPrivateKey {
    /// Creates a private master key (extended format) from the given seed.
    pub fn create_master_key(seed: [u8; 64], testnet: bool) -> ExtendedPrivateKey {
        let mut engine = hmac::HmacEngine::<sha512::Hash>::new(b"Bitcoin seed");
        engine.input(&seed);
        let seed_hash = hmac::Hmac::from_engine(engine).into_inner();
        let (private_key, chain_code) = seed_hash.split_at(seed_hash.len() / 2);

        let depth = 0x00;
        let fingerprint = [0x00, 0x00, 0x00, 0x00];
        let child_number = [0x00, 0x00, 0x00, 0x00];
        let chain_code: [u8; 32] = chain_code.try_into().unwrap();
        let key_data: [u8; 32] = private_key.try_into().unwrap();

        ExtendedPrivateKey {
            testnet,
            depth,
            fingerprint,
            child_number,
            chain_code,
            key_data,
        }
    }

    /// Imports a key from the Base58Check format.
    pub fn import_key_from_base58_check(key: &str) -> Result<ExtendedPrivateKey, ImportKeyError> {
        let bytes = base58::decode(key);
        if let None = bytes {
            return Err(ImportKeyError::new("Invalid character"));
        }
        let bytes = bytes.unwrap();
        let version = &bytes[..4];
        if version != [0x04, 0x88, 0xAD, 0xE4] && version != [0x04, 0x35, 0x83, 0x94] {
            return Err(ImportKeyError::new("Invalid version bytes"));
        }
        let depth = bytes[4];
        let fingerprint = &bytes[5..9];
        let child_number = &bytes[9..13];
        let chain_code = &bytes[13..45];
        // skip 0x00 byte
        let key_data = &bytes[46..78];
        let checksum_target = &bytes[78..];

        let checksum = sha256::Hash::hash(&bytes[..78]);
        let checksum = sha256::Hash::hash(&checksum);
        let checksum = &checksum.into_inner()[..4];
        if checksum != checksum_target {
            return Err(ImportKeyError::new("Invalid checksum"));
        }
        Ok(ExtendedPrivateKey {
            testnet: version == [0x04, 0x35, 0x83, 0x94],
            depth,
            fingerprint: fingerprint.try_into().unwrap(),
            child_number: child_number.try_into().unwrap(),
            chain_code: chain_code.try_into().unwrap(),
            key_data: key_data.try_into().unwrap(),
        })
    }

    /// Exports the key to the Base58Check format.
    pub fn to_base58_check(&self) -> String {
        let mut key_bytes = Vec::with_capacity(78);
        if self.testnet {
            key_bytes.extend([0x04, 0x35, 0x83, 0x94]);
        } else {
            key_bytes.extend([0x04, 0x88, 0xAD, 0xE4]);
        }
        key_bytes.push(self.depth);
        key_bytes.extend(self.fingerprint);
        key_bytes.extend(self.child_number);
        key_bytes.extend(self.chain_code);
        key_bytes.push(0x00);
        key_bytes.extend(self.key_data);

        let checksum = sha256::Hash::hash(&key_bytes);
        let checksum = sha256::Hash::hash(&checksum);
        key_bytes.extend(&checksum[..4]);
        base58::encode(&key_bytes)
    }

    /// Derives a child key (child key derivation function).
    pub fn derive_child_key(
        &self,
        num: u32,
        hardened: bool,
    ) -> Result<ExtendedPrivateKey, ChildKeyDeriveError> {
        if num >= 2u32.pow(31) {
            return Err(ChildKeyDeriveError::new("Child number out of bounds"));
        }
        if self.depth == 0xFF {
            return Err(ChildKeyDeriveError::new("Maximum depth reached"));
        }
        let hash_value = if hardened {
            // hardened child
            let i: u32 = 2u32.pow(31) + num;
            let mut engine = hmac::HmacEngine::<sha512::Hash>::new(&self.chain_code);
            let mut data = Vec::with_capacity(37);
            data.push(0x00);
            data.extend(&self.key_data);
            data.extend(i.to_be_bytes());
            engine.input(&data);
            hmac::Hmac::from_engine(engine).into_inner()
        } else {
            // normal child
            let i: u32 = num;
            let secp = Secp256k1::new();
            let secret_key =
                SecretKey::from_slice(&self.key_data).expect("32 bytes, within curve order");
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);

            let mut engine = hmac::HmacEngine::<sha512::Hash>::new(&self.chain_code);
            let mut data = Vec::with_capacity(37);
            data.extend(public_key.serialize());
            data.extend(i.to_be_bytes());
            engine.input(&data);
            hmac::Hmac::from_engine(engine).into_inner()
        };
        let (il, child_chain_code) = hash_value.split_at(hash_value.len() / 2);
        let parse_il = BigUint::from_bytes_be(il);
        let k_par = BigUint::from_bytes_be(&self.key_data);
        let n = BigUint::from_bytes_be(&secp256k1::constants::CURVE_ORDER);
        let child_key = (parse_il.clone() + k_par) % n.clone();
        let child_key_data = child_key.to_bytes_be();
        if parse_il >= n || child_key == BigUint::from(0u8) {
            return self.derive_child_key(num + 1, hardened);
        }
        // normal child range [0, 2^31 - 1]
        // hardened child range [2^31, 2^32 - 1]
        let i: u32 = if hardened { 2u32.pow(31) + num } else { num };
        Ok(ExtendedPrivateKey {
            testnet: self.testnet,
            depth: self.depth + 0x01,
            fingerprint: self.get_fingerprint(),
            child_number: i.to_be_bytes().try_into().unwrap(),
            chain_code: child_chain_code.try_into().unwrap(),
            key_data: child_key_data.try_into().unwrap(),
        })
    }

    /// Computes the fingerprint of the key.
    pub fn get_fingerprint(&self) -> [u8; 4] {
        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_slice(&self.key_data).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let hash = sha256::Hash::hash(&public_key.serialize()).to_vec();
        let hash = ripemd160::Hash::hash(&hash);
        hash[..4].try_into().unwrap()
    }

    pub fn derive_public_key(&self) -> ExtendedPublicKey {
        ExtendedPublicKey::derive_public_key(&self)
    }
}

impl ExtendedPublicKey {
    /// Derives the public key (extended format) from the given private key.
    pub fn derive_public_key(private_key: &ExtendedPrivateKey) -> ExtendedPublicKey {
        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_slice(&private_key.key_data).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let key_data = public_key.serialize();

        ExtendedPublicKey {
            testnet: private_key.testnet,
            depth: private_key.depth.clone(),
            fingerprint: private_key.fingerprint.clone(),
            child_number: private_key.child_number.clone(),
            chain_code: private_key.chain_code.clone(),
            key_data: key_data.to_vec(),
        }
    }

    /// Imports a key from the Base58Check format.
    pub fn import_key_from_base58_check(key: &str) -> Result<ExtendedPublicKey, ImportKeyError> {
        let bytes = base58::decode(key);
        if let None = bytes {
            return Err(ImportKeyError::new("Invalid character"));
        }
        let bytes = bytes.unwrap();
        let version = &bytes[..4];
        if version != [0x04, 0x88, 0xB2, 0x1E] && version != [0x04, 0x35, 0x87, 0xCF] {
            return Err(ImportKeyError::new("Invalid version bytes"));
        }
        let depth = bytes[4];
        let fingerprint = &bytes[5..9];
        let child_number = &bytes[9..13];
        let chain_code = &bytes[13..45];
        let key_data = &bytes[45..78];
        let checksum_target = &bytes[78..];

        let checksum = sha256::Hash::hash(&bytes[..78]);
        let checksum = sha256::Hash::hash(&checksum);
        let checksum = &checksum.into_inner()[..4];
        if checksum != checksum_target {
            return Err(ImportKeyError::new("Invalid checksum"));
        }
        if let Err(_) = PublicKey::from_slice(key_data) {
            // check that the x-coordinate corresponds to a point on the curve
            return Err(ImportKeyError::new("Invalid key"));
        }
        Ok(ExtendedPublicKey {
            testnet: version == [0x04, 0x35, 0x87, 0xCF],
            depth,
            fingerprint: fingerprint.try_into().unwrap(),
            child_number: child_number.try_into().unwrap(),
            chain_code: chain_code.try_into().unwrap(),
            key_data: key_data.try_into().unwrap(),
        })
    }

    /// Exports the key to the Base58Check format.
    pub fn to_base58_check(&self) -> String {
        let mut key_bytes = Vec::with_capacity(78);

        if self.testnet {
            key_bytes.extend([0x04, 0x35, 0x87, 0xCF]);
        } else {
            key_bytes.extend([0x04, 0x88, 0xB2, 0x1E]);
        }
        key_bytes.push(self.depth);
        key_bytes.extend(self.fingerprint);
        key_bytes.extend(self.child_number);
        key_bytes.extend(self.chain_code);
        key_bytes.extend(&self.key_data);

        let checksum = sha256::Hash::hash(&key_bytes);
        let checksum = sha256::Hash::hash(&checksum);
        key_bytes.extend(&checksum[..4]);
        base58::encode(&key_bytes)
    }

    /// Derives a child key (child key derivation function).
    pub fn derive_child_key(&self, num: u32) -> Result<ExtendedPublicKey, ChildKeyDeriveError> {
        if num >= 2u32.pow(31) {
            return Err(ChildKeyDeriveError::new("Child number out of bounds"));
        }
        if self.depth == 0xFF {
            return Err(ChildKeyDeriveError::new("Maximum depth reached"));
        }
        let i: u32 = num;
        let public_key = PublicKey::from_slice(&self.key_data).unwrap();

        let mut engine = hmac::HmacEngine::<sha512::Hash>::new(&self.chain_code);
        let mut data = Vec::with_capacity(37);
        data.extend(public_key.serialize());
        data.extend(i.to_be_bytes());
        engine.input(&data);
        let hash_value = hmac::Hmac::from_engine(engine).into_inner();

        let (il, child_chain_code) = hash_value.split_at(hash_value.len() / 2);

        let parse_il = BigUint::from_bytes_be(il);
        let n = BigUint::from_bytes_be(&secp256k1::constants::CURVE_ORDER);
        if parse_il >= n {
            return self.derive_child_key(num + 1);
        }
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&il).expect("32 bytes, within curve order");
        let point_il = PublicKey::from_secret_key(&secp, &secret_key);
        let point_child = point_il.combine(&public_key);
        let point_child = match point_child {
            Ok(point_child) => point_child,
            Err(_) => return self.derive_child_key(num + 1),
        };
        let child_key_data = point_child.serialize();
        Ok(ExtendedPublicKey {
            testnet: self.testnet,
            depth: self.depth + 0x01,
            fingerprint: self.get_fingerprint(),
            child_number: i.to_be_bytes().try_into().unwrap(),
            chain_code: child_chain_code.try_into().unwrap(),
            key_data: child_key_data.to_vec(),
        })
    }

    /// Computes the fingerprint of the key.
    pub fn get_fingerprint(&self) -> [u8; 4] {
        let public_key = PublicKey::from_slice(&self.key_data).unwrap();
        let hash = sha256::Hash::hash(&public_key.serialize()).to_vec();
        let hash = ripemd160::Hash::hash(&hash);
        hash[..4].try_into().unwrap()
    }

    /// Returns the address of the public key.
    pub fn get_address(&self) -> Address {
        Address::create(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::keys::bip32::{ExtendedPrivateKey, ExtendedPublicKey};

    #[test]
    fn test_create_master_key_private() {
        let seed = [
            91, 86, 196, 23, 48, 63, 170, 63, 203, 167, 229, 116, 0, 225, 32, 160, 202, 131, 236,
            90, 79, 201, 255, 186, 117, 127, 190, 99, 251, 215, 122, 137, 161, 163, 190, 76, 103,
            25, 111, 87, 195, 154, 136, 183, 99, 115, 115, 56, 145, 191, 171, 161, 110, 210, 122,
            129, 60, 238, 212, 152, 128, 76, 5, 112,
        ];
        let master_key = ExtendedPrivateKey::create_master_key(seed, false).to_base58_check();
        let target_master_key = String::from("xprv9s21ZrQH143K3t4UZrNgeA3w861fwjYLaGwmPtQyPMmzshV2owVpfBSd2Q7YsHZ9j6i6ddYjb5PLtUdMZn8LhvuCVhGcQntq5rn7JVMqnie");
        assert_eq!(master_key, target_master_key);
    }

    #[test]
    fn test_create_master_key_private_testnet() {
        let seed = [
            91, 86, 196, 23, 48, 63, 170, 63, 203, 167, 229, 116, 0, 225, 32, 160, 202, 131, 236,
            90, 79, 201, 255, 186, 117, 127, 190, 99, 251, 215, 122, 137, 161, 163, 190, 76, 103,
            25, 111, 87, 195, 154, 136, 183, 99, 115, 115, 56, 145, 191, 171, 161, 110, 210, 122,
            129, 60, 238, 212, 152, 128, 76, 5, 112,
        ];
        let master_key = ExtendedPrivateKey::create_master_key(seed, true).to_base58_check();
        let target_master_key = String::from("tprv8ZgxMBicQKsPehJ1EREBoofvSDRtBFaLuprtGJqRsLGUfJE7oJqaAvp4waHCsewU6YEsdjAVkRy9MLB6gzUHWzAo2LUv59cszxXXk7p7vv3");
        assert_eq!(master_key, target_master_key);
    }

    #[test]
    fn test_import_key_from_base58_check_private() {
        let key_base58_check = "xprv9tyUQV64JT5qs3RSTJkXCWKMyUgoQp7F3hA1xzG6ZGu6u6Q9VMNjGr67Lctvy5P8oyaYAL9CAWrUE9i6GoNMKUga5biW6Hx4tws2six3b9c";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(key_base58_check).unwrap();
        let export_key_base58_check = private_key.to_base58_check();
        assert_eq!(export_key_base58_check, key_base58_check);
    }

    #[test]
    fn test_derive_public_key() {
        let private_key_b58 = "xprv9tyUQV64JT5qs3RSTJkXCWKMyUgoQp7F3hA1xzG6ZGu6u6Q9VMNjGr67Lctvy5P8oyaYAL9CAWrUE9i6GoNMKUga5biW6Hx4tws2six3b9c";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let public_key = ExtendedPublicKey::derive_public_key(&private_key);
        let public_key_b58_check = public_key.to_base58_check();
        let public_key_target = String::from("xpub67xpozcx8pe95XVuZLHXZeG6XWXHpGq6Qv5cmNfi7cS5mtjJ2tgypeQbBs2UAR6KECeeMVKZBPLrtJunSDMstweyLXhRgPxdp14sk9tJPW9");
        assert_eq!(public_key_b58_check, public_key_target);
    }

    #[test]
    fn test_import_public_key() {
        let public_key_b58 = "xpub67xpozcx8pe95XVuZLHXZeG6XWXHpGq6Qv5cmNfi7cS5mtjJ2tgypeQbBs2UAR6KECeeMVKZBPLrtJunSDMstweyLXhRgPxdp14sk9tJPW9";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        assert_eq!(public_key.to_base58_check(), public_key_b58);
    }

    #[test]
    fn test_derive_private_child_key_1() {
        let private_key_b58 = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(0, true).unwrap();
        let target = "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7";
        assert_eq!(private_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_private_child_key_2() {
        let private_key_b58 = "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(1, false).unwrap();
        let target = "xprv9wTYmMFdV23N2TdNG573QoEsfRrWKQgWeibmLntzniatZvR9BmLnvSxqu53Kw1UmYPxLgboyZQaXwTCg8MSY3H2EU4pWcQDnRnrVA1xe8fs";
        assert_eq!(private_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_private_child_key_3() {
        let private_key_b58 = "xprv9wTYmMFdV23N2TdNG573QoEsfRrWKQgWeibmLntzniatZvR9BmLnvSxqu53Kw1UmYPxLgboyZQaXwTCg8MSY3H2EU4pWcQDnRnrVA1xe8fs";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(2, true).unwrap();
        let target = "xprv9z4pot5VBttmtdRTWfWQmoH1taj2axGVzFqSb8C9xaxKymcFzXBDptWmT7FwuEzG3ryjH4ktypQSAewRiNMjANTtpgP4mLTj34bhnZX7UiM";
        assert_eq!(private_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_private_child_key_4() {
        let private_key_b58 = "xprv9z4pot5VBttmtdRTWfWQmoH1taj2axGVzFqSb8C9xaxKymcFzXBDptWmT7FwuEzG3ryjH4ktypQSAewRiNMjANTtpgP4mLTj34bhnZX7UiM";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(2, false).unwrap();
        let target = "xprvA2JDeKCSNNZky6uBCviVfJSKyQ1mDYahRjijr5idH2WwLsEd4Hsb2Tyh8RfQMuPh7f7RtyzTtdrbdqqsunu5Mm3wDvUAKRHSC34sJ7in334";
        assert_eq!(private_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_private_child_key_5() {
        let private_key_b58 = "xprvA2JDeKCSNNZky6uBCviVfJSKyQ1mDYahRjijr5idH2WwLsEd4Hsb2Tyh8RfQMuPh7f7RtyzTtdrbdqqsunu5Mm3wDvUAKRHSC34sJ7in334";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(1000000000, false).unwrap();
        let target = "xprvA41z7zogVVwxVSgdKUHDy1SKmdb533PjDz7J6N6mV6uS3ze1ai8FHa8kmHScGpWmj4WggLyQjgPie1rFSruoUihUZREPSL39UNdE3BBDu76";
        assert_eq!(private_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_public_child_key_1() {
        let public_key_b58 = "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let public_key_child = public_key.derive_child_key(0).unwrap();
        let target = "xpub69H7F5d8KSRgmmdJg2KhpAK8SR3DjMwAdkxj3ZuxV27CprR9LgpeyGmXUbC6wb7ERfvrnKZjXoUmmDznezpbZb7ap6r1D3tgFxHmwMkQTPH";
        assert_eq!(public_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_derive_public_child_key() {
        let private_key_b58 = "xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let private_key_child = private_key.derive_child_key(0, false).unwrap();

        let public_key_b58 = "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let public_key_child = public_key.derive_child_key(0).unwrap();

        assert_eq!(
            public_key_child.to_base58_check(),
            private_key_child.derive_public_key().to_base58_check()
        );
    }

    #[test]
    fn test_derive_private_and_public_key() {
        let public_key_b58 = "xpub6ASAVgeehLbnwdqV6UKMHVzgqAG8Gr6riv3Fxxpj8ksbH9ebxaEyBLZ85ySDhKiLDBrQSARLq1uNRts8RuJiHjaDMBU4Zn9h8LZNnBC5y4a";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let public_key_child = public_key.derive_child_key(1).unwrap();
        let target = "xpub6DF8uhdarytz3FWdA8TvFSvvAh8dP3283MY7p2V4SeE2wyWmG5mg5EwVvmdMVCQcoNJxGoWaU9DCWh89LojfZ537wTfunKau47EL2dhHKon";
        assert_eq!(public_key_child.to_base58_check(), String::from(target));
    }

    #[test]
    fn test_get_address_mainnet() {
        let public_key_b58 = "xpub6AHA9hZDN11k2ijHMeS5QqHx2KP9aMBRhTDqANMnwVtdyw2TDYRmF8PjpvwUFcL1Et8Hj59S3gTSMcUQ5gAqTz3Wd8EsMTmF3DChhqPQBnU";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let target = String::from("1Nro9WkpaKm9axmcfPVp79dAJU1Gx7VmMZ");
        assert_eq!(public_key.get_address().to_string(), target);

        let public_key_b58 = "xpub6AE6VDqYzHBZsv9zt9uXBZ1Hzw9yLuGLJzYMAgxjvBGfTV69EWKbdxs8CD4xKBoGYFRrBLGvkgrutpFV1ygCT24Ch4RLf3KezhmfkG6EC8t";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let target = String::from("15dF9N4tLmB5X6pYiZbsPTXMBXJvCwC3wg");
        assert_eq!(public_key.get_address().to_string(), target);
    }

    #[test]
    fn test_get_address_testnet() {
        let public_key_b58 = "tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT8J2NDm1D8pfirmrQqLYRFFmrhtxHWiYq9t73jtzZhi4dtsPkV";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let target = String::from("mxoePtNPAGTUCe7814kjGx9zHddaERxu4r");
        assert_eq!(public_key.get_address().to_string(), target);
    }
}
