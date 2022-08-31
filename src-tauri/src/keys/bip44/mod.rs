//! Implements a logical hierarchy for deterministic wallets as described
//! in [BIP-44](https://en.bitcoin.it/wiki/BIP_0044).
use lazy_static::lazy_static;
use std::collections::HashMap;

pub mod private_hierarchy;
pub mod public_hierarchy;
pub use private_hierarchy::{Keypair, MasterPrivateKey};
pub use public_hierarchy::MasterPublicKey;

pub static BITCOIN_INDEX: u32 = 0;
pub static BITCOIN_TESTNET_INDEX: u32 = 1;

lazy_static! {
    static ref COIN_TYPE_NAMES: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "Bitcoin");
        m.insert(1, "Bitcoin Testnet");
        m
    };
}

#[cfg(test)]
mod tests {
    use crate::keys::{
        bip32::ExtendedPrivateKey,
        bip44::{
            private_hierarchy::MasterPrivateKey, public_hierarchy::MasterPublicKey, BITCOIN_INDEX,
        },
    };

    #[test]
    fn test_derive_private_and_public_key() {
        let private_key_b58 = "xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U";
        let private_key =
            ExtendedPrivateKey::import_key_from_base58_check(private_key_b58).unwrap();
        let mut master_private_key = MasterPrivateKey::create_from_key(private_key);
        let mut master_public_key = MasterPublicKey::create_from_key(&master_private_key);
        let account = master_private_key.add_account(BITCOIN_INDEX);
        master_public_key.add_account(BITCOIN_INDEX, &account);
        let child_keypair =
            master_private_key.new_change_keypair(BITCOIN_INDEX, account.index, None);
        let change_key = master_public_key.new_change_key(BITCOIN_INDEX, account.index);
        assert_eq!(
            change_key.to_base58_check(),
            child_keypair.public_key.to_base58_check()
        );
    }
}
