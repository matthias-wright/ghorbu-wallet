//! Implements a Base58Check address.
use crate::keys::bip32::ExtendedPublicKey;
use crate::keys::error::ParseAddressError;
use crate::utils::{base58, hex};
use bitcoin_hashes::{ripemd160, sha256, Hash};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Address {
    pub testnet: bool,
    pub hash160: [u8; 20],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleAddress {
    pub address: String,
    pub used: bool,
}

impl Address {
    /// Creates an address.
    pub fn create(public_key: &ExtendedPublicKey) -> Address {
        let sha_256 = sha256::Hash::hash(&public_key.key_data);
        let hash160 = ripemd160::Hash::hash(&sha_256);
        Address {
            testnet: public_key.testnet,
            hash160: hash160.into_inner(),
        }
    }

    /// Returns the address in hex format.
    pub fn get_h160(&self) -> String {
        hex::bytes_to_hex(&self.hash160)
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        let mut hash160_prefix = Vec::with_capacity(25);
        if self.testnet {
            hash160_prefix.push(0x6f);
        } else {
            hash160_prefix.push(0x00);
        }
        hash160_prefix.extend(&self.hash160);
        let checksum = sha256::Hash::hash(&hash160_prefix);
        let checksum = sha256::Hash::hash(&checksum);
        hash160_prefix.extend(&checksum[..4]);
        base58::encode(&hash160_prefix)
    }
}

impl FromStr for Address {
    type Err = ParseAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = base58::decode(s);
        if let None = bytes {
            return Err(ParseAddressError::new("Invalid character"));
        }
        let bytes = bytes.unwrap();
        if bytes.len() != 25 {
            return Err(ParseAddressError::new("Invalid length"));
        }
        let prefix = &bytes[0];
        if *prefix != 0x00 && *prefix != 0x6f {
            return Err(ParseAddressError::new("Invalid prefix"));
        }
        let hash = &bytes[1..bytes.len() - 4];
        let checksum_target = &bytes[bytes.len() - 4..];
        let checksum = sha256::Hash::hash(&bytes[..bytes.len() - 4]);
        let checksum = sha256::Hash::hash(&checksum);
        if checksum_target != &checksum[..4] {
            return Err(ParseAddressError::new("Checksum failed"));
        }
        Ok(Address {
            testnet: *prefix == 0x6f,
            hash160: hash.try_into().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::keys::address::Address;
    use crate::keys::bip32::ExtendedPublicKey;
    use std::str::FromStr;

    #[test]
    fn test_address_export() {
        let public_key_b58 = "xpub6AHA9hZDN11k2ijHMeS5QqHx2KP9aMBRhTDqANMnwVtdyw2TDYRmF8PjpvwUFcL1Et8Hj59S3gTSMcUQ5gAqTz3Wd8EsMTmF3DChhqPQBnU";
        let public_key = ExtendedPublicKey::import_key_from_base58_check(public_key_b58).unwrap();
        let address = public_key.get_address();
        let address_target = String::from("1Nro9WkpaKm9axmcfPVp79dAJU1Gx7VmMZ");
        assert_eq!(address.to_string(), address_target);
    }

    #[test]
    fn test_address_import() {
        let address_str = "1vFgGCtnBLEobbQMEbz13Vw6RF64H2SYD";
        let address = Address::from_str(address_str).unwrap();
        assert_eq!(address.to_string(), address_str.to_string());
    }
}
