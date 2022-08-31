//! Implements a logical hierarchy for deterministic wallets as described
//! in [BIP-44](https://en.bitcoin.it/wiki/BIP_0044).
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    private_hierarchy::{Account, Change, CoinType, MasterPrivateKey, Purpose},
    BITCOIN_INDEX, BITCOIN_TESTNET_INDEX,
};
use crate::keys::address::Address;
use crate::keys::bip32::ExtendedPublicKey;

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterPublicKey {
    pub public_key: ExtendedPublicKey,
    pub purpose: PublicPurpose,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicPurpose {
    pub index: u32,
    pub public_key: ExtendedPublicKey,
    pub coin_types: BTreeMap<u32, PublicCoinType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicCoinType {
    pub index: u32,
    pub name: String,
    pub public_key: ExtendedPublicKey,
    pub accounts: BTreeMap<u32, PublicAccount>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicAccount {
    pub index: u32,
    pub public_key: ExtendedPublicKey,
    pub external_chain: PublicChange, // used for receiving payments
    pub internal_chain: PublicChange, // used for change
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicChange {
    pub index: u32,
    pub public_key: ExtendedPublicKey,
    pub keys: BTreeMap<u32, ExtendedPublicKey>,
}

impl MasterPublicKey {
    /// Creates a public key hierarchy from the given master public key.
    pub fn create_from_key(master_private_key: &MasterPrivateKey) -> MasterPublicKey {
        MasterPublicKey {
            public_key: master_private_key.private_key.derive_public_key(),
            purpose: PublicPurpose::create(&master_private_key.purpose),
        }
    }

    /// Adds an account for the specified coin type.
    pub fn add_account(&mut self, coin_type_index: u32, account: &Account) {
        self.purpose.add_account(coin_type_index, account);
    }

    /// Creates and returns a new public key for return transaction change.
    pub fn new_change_key(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
    ) -> ExtendedPublicKey {
        self.purpose.new_change_key(coin_type_index, account_index)
    }

    /// Creates and returns a new public key for receiving payments.
    pub fn new_receive_key(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
    ) -> ExtendedPublicKey {
        self.purpose.new_receive_key(coin_type_index, account_index)
    }

    /// Returns all receive addresses for the specified account.
    pub fn get_all_addresses(&self, coin_type_index: u32, account_index: u32) -> Vec<Address> {
        let receive_addresses = self.get_all_receive_addresses(coin_type_index, account_index);
        let change_addresses = self.get_all_change_addresses(coin_type_index, account_index);
        receive_addresses
            .into_iter()
            .chain(change_addresses.into_iter())
            .collect()
    }

    /// Returns all receive addresses for the specified account.
    pub fn get_all_receive_addresses(
        &self,
        coin_type_index: u32,
        account_index: u32,
    ) -> Vec<Address> {
        let coin_type = self.purpose.coin_types.get(&coin_type_index).unwrap();
        let account = coin_type.accounts.get(&account_index).unwrap();
        account
            .external_chain
            .keys
            .iter()
            .map(|(_, key)| key.get_address())
            .rev()
            .collect()
    }

    /// Returns all change addresses for the specified account.
    pub fn get_all_change_addresses(
        &self,
        coin_type_index: u32,
        account_index: u32,
    ) -> Vec<Address> {
        let coin_type = self.purpose.coin_types.get(&coin_type_index).unwrap();
        let account = coin_type.accounts.get(&account_index).unwrap();
        account
            .internal_chain
            .keys
            .iter()
            .map(|(_, key)| key.get_address())
            .rev()
            .collect()
    }

    /// Serializes the key hierarchy to a vector of bytes.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Loads the key hierarchy from the specified path.
    pub fn load(bytes: &[u8]) -> MasterPublicKey {
        bincode::deserialize(bytes).unwrap()
    }

    pub fn jsonify(&self) -> String {
        format!("{{\"purpose\": {}}}", self.purpose.jsonify())
    }
}

impl PublicPurpose {
    fn create(purpose: &Purpose) -> PublicPurpose {
        PublicPurpose {
            index: purpose.index,
            public_key: purpose.private_key.derive_public_key(),
            coin_types: purpose
                .coin_types
                .iter()
                .map(|(index, coin_type)| (*index, PublicCoinType::create(coin_type)))
                .collect(),
        }
    }

    /// Creates and returns a new public key for return transaction change.
    pub fn new_change_key(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
    ) -> ExtendedPublicKey {
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        let account = coin_type.accounts.get_mut(&account_index).unwrap();
        account.new_change_key()
    }

    /// Creates and returns a new public key for receiving payments.
    pub fn new_receive_key(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
    ) -> ExtendedPublicKey {
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        let account = coin_type.accounts.get_mut(&account_index).unwrap();
        account.new_receive_key()
    }

    pub fn add_account(&mut self, coin_type_index: u32, account: &Account) {
        assert!(coin_type_index == BITCOIN_INDEX || coin_type_index == BITCOIN_TESTNET_INDEX);
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        let public_account = PublicAccount::create(account);
        coin_type.accounts.insert(account.index, public_account);
    }

    fn jsonify(&self) -> String {
        let coin_types: String = self
            .coin_types
            .iter()
            .map(|(_, coin_type)| coin_type.jsonify())
            .collect::<Vec<String>>()
            .join(",");
        format!(
            "{{\"index\": {}, \"coin_types\": [{}]}}",
            self.index, coin_types
        )
    }
}

impl PublicCoinType {
    /// Creates a new coin type.
    fn create(coin_type: &CoinType) -> PublicCoinType {
        PublicCoinType {
            index: coin_type.index,
            name: coin_type.name.clone(),
            public_key: coin_type.private_key.derive_public_key(),
            accounts: coin_type
                .accounts
                .iter()
                .map(|(index, account)| (*index, PublicAccount::create(account)))
                .collect(),
        }
    }

    fn jsonify(&self) -> String {
        let accounts: String = self
            .accounts
            .iter()
            .map(|(_, account)| account.jsonify())
            .collect::<Vec<String>>()
            .join(",");
        format!(
            "{{\"index\": {}, \"name\": \"{}\", \"accounts\": [{}]}}",
            self.index, self.name, accounts
        )
    }
}

impl PublicAccount {
    /// Creates a new account.
    fn create(account: &Account) -> PublicAccount {
        PublicAccount {
            index: account.index,
            public_key: account.private_key.derive_public_key(),
            external_chain: PublicChange::create(&account.external_chain),
            internal_chain: PublicChange::create(&account.internal_chain),
        }
    }

    /// Creates and returns a new public key for return transaction change.
    fn new_change_key(&mut self) -> ExtendedPublicKey {
        self.internal_chain.new_key()
    }

    /// Creates and returns a new public key for receiving payments.
    fn new_receive_key(&mut self) -> ExtendedPublicKey {
        self.external_chain.new_key()
    }

    fn jsonify(&self) -> String {
        format!("{{\"index\": {}}}", self.index)
    }
}

impl PublicChange {
    /// Creates a new change type.
    fn create(change: &Change) -> PublicChange {
        PublicChange {
            index: change.index,
            public_key: change.private_key.derive_public_key(),
            keys: change
                .keys
                .iter()
                .map(|(index, keypair)| (*index, keypair.public_key.clone()))
                .collect(),
        }
    }

    /// Creates and returns a new public key.
    fn new_key(&mut self) -> ExtendedPublicKey {
        let index = if self.keys.len() == 0 {
            0
        } else {
            self.keys.iter().max_by(|a, b| a.0.cmp(&b.0)).unwrap().0 + 1
        };
        let child_public_key = self.public_key.derive_child_key(index).unwrap();
        self.keys.insert(index, child_public_key.clone());
        child_public_key
    }
}
