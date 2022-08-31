//! Implements a logical hierarchy for deterministic wallets as described
//! in [BIP-44](https://en.bitcoin.it/wiki/BIP_0044).
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use super::{BITCOIN_INDEX, BITCOIN_TESTNET_INDEX, COIN_TYPE_NAMES};
use crate::encryption;
use crate::keys::bip32::{ExtendedPrivateKey, ExtendedPublicKey};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MasterPrivateKey {
    pub private_key: ExtendedPrivateKey,
    pub purpose: Purpose,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Purpose {
    pub index: u32,
    pub private_key: ExtendedPrivateKey,
    pub coin_types: BTreeMap<u32, CoinType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoinType {
    pub index: u32,
    pub name: String,
    pub private_key: ExtendedPrivateKey,
    pub accounts: BTreeMap<u32, Account>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub index: u32,
    pub private_key: ExtendedPrivateKey,
    pub external_chain: Change, // used for receiving payments
    pub internal_chain: Change, // used for change
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Change {
    pub index: u32,
    pub private_key: ExtendedPrivateKey,
    pub keys: BTreeMap<u32, Keypair>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keypair {
    pub private_key: ExtendedPrivateKey,
    pub public_key: ExtendedPublicKey,
}

impl MasterPrivateKey {
    /// Creates a private key hierarchy from the given master private key.
    pub fn create_from_key(private_key: ExtendedPrivateKey) -> MasterPrivateKey {
        let purpose = Purpose::create(&private_key);
        MasterPrivateKey {
            private_key,
            purpose,
        }
    }

    /// Creates a private key hierarchy from the given seed.
    pub fn create_from_seed(seed: [u8; 64]) -> MasterPrivateKey {
        let private_key = ExtendedPrivateKey::create_master_key(seed, true);
        let purpose = Purpose::create(&private_key);
        MasterPrivateKey {
            private_key,
            purpose,
        }
    }

    /// Adds an account for the specified coin type.
    pub fn add_account(&mut self, coin_type_index: u32) -> Account {
        self.purpose.add_account(coin_type_index)
    }

    /// Returns all key pairs for the specified account.
    pub fn get_all_keypairs(&self, coin_type_index: u32, account_index: u32) -> Vec<Keypair> {
        let receive_keypairs = self.get_all_receive_keypairs(coin_type_index, account_index);
        let change_keypairs = self.get_all_change_keypairs(coin_type_index, account_index);
        receive_keypairs
            .into_iter()
            .chain(change_keypairs.into_iter())
            .collect()
    }

    /// Returns all change key pairs for the specified account.
    pub fn get_all_change_keypairs(
        &self,
        coin_type_index: u32,
        account_index: u32,
    ) -> Vec<Keypair> {
        let coin_type = self.purpose.coin_types.get(&coin_type_index).unwrap();
        let account = coin_type.accounts.get(&account_index).unwrap();
        account
            .internal_chain
            .keys
            .iter()
            .map(|(_, keypair)| keypair.clone())
            .collect()
    }

    /// Returns all receive key pairs for the specified account.
    pub fn get_all_receive_keypairs(
        &self,
        coin_type_index: u32,
        account_index: u32,
    ) -> Vec<Keypair> {
        let coin_type = self.purpose.coin_types.get(&coin_type_index).unwrap();
        let account = coin_type.accounts.get(&account_index).unwrap();
        account
            .external_chain
            .keys
            .iter()
            .map(|(_, keypair)| keypair.clone())
            .collect()
    }

    /// Creates and returns a new keypair for return transaction change.
    pub fn new_change_keypair(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
        key_index: Option<u32>,
    ) -> Keypair {
        self.purpose
            .new_change_keypair(coin_type_index, account_index, key_index)
    }

    /// Creates and returns a new keypair for receiving payments.
    pub fn new_receive_keypair(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
        key_index: Option<u32>,
    ) -> Keypair {
        self.purpose
            .new_receive_keypair(coin_type_index, account_index, key_index)
    }

    /// Returns the specified private key. If this key does not exist, it is derived.
    pub fn get_private_key(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
        internal: bool,
        key_index: u32,
    ) -> Option<ExtendedPrivateKey> {
        let coin_type = match self.purpose.coin_types.get_mut(&coin_type_index) {
            Some(coin_type) => coin_type,
            None => return None,
        };
        let account = match coin_type.accounts.get_mut(&account_index) {
            Some(account) => account,
            None => return None,
        };
        let change = if internal {
            &account.internal_chain
        } else {
            &account.external_chain
        };
        let keypair = match change.keys.get(&key_index) {
            Some(keypair) => keypair.clone(),
            None => {
                if internal {
                    self.new_change_keypair(coin_type_index, account_index, Some(key_index))
                } else {
                    self.new_receive_keypair(coin_type_index, account_index, Some(key_index))
                }
            }
        };
        Some(keypair.private_key)
    }

    /// Saves the key hierarchy to the specified path.
    pub fn save<P: AsRef<Path>>(&self, path: P, password: String) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();
        let master_key_encoded: Vec<u8> = bincode::serialize(self)?;
        let master_key_encrypted = encryption::encrypt(master_key_encoded, password);
        let mut file = File::create(path)?;
        file.write_all(&master_key_encrypted)?;
        Ok(())
    }

    /// Loads the key hierarchy from the specified path.
    pub fn load<P: AsRef<Path>>(
        path: P,
        password: String,
    ) -> Result<MasterPrivateKey, Box<dyn Error>> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        let buffer_decrypted = encryption::decrypt(buffer, password)?;
        let master_key: MasterPrivateKey = bincode::deserialize(&buffer_decrypted[..])?;
        Ok(master_key)
    }
}

impl Purpose {
    /// Creates a new purpose.
    fn create(private_key: &ExtendedPrivateKey) -> Purpose {
        let child_key = private_key.derive_child_key(44, true).unwrap();
        let coin_types = COIN_TYPE_NAMES
            .iter()
            .map(|(index, _)| (*index, CoinType::create(*index, &child_key)))
            .collect();
        Purpose {
            index: 44,
            private_key: child_key,
            coin_types,
        }
    }

    /// Adds an account for the specified coin type.
    fn add_account(&mut self, coin_type_index: u32) -> Account {
        assert!(coin_type_index == BITCOIN_INDEX || coin_type_index == BITCOIN_TESTNET_INDEX);
        if !self.coin_types.contains_key(&coin_type_index) {
            self.add_coin_type(coin_type_index);
        }
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        coin_type.add_account()
    }

    /// Creates and returns a new keypair for return transaction change.
    fn new_change_keypair(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
        key_index: Option<u32>,
    ) -> Keypair {
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        let account = coin_type.accounts.get_mut(&account_index).unwrap();
        account.new_change_keypair(key_index)
    }

    /// Creates and returns a new keypair for receiving payments.
    fn new_receive_keypair(
        &mut self,
        coin_type_index: u32,
        account_index: u32,
        key_index: Option<u32>,
    ) -> Keypair {
        let coin_type = self.coin_types.get_mut(&coin_type_index).unwrap();
        let account = coin_type.accounts.get_mut(&account_index).unwrap();
        account.new_receive_keypair(key_index)
    }

    /// Adds a new coin type.
    fn add_coin_type(&mut self, index: u32) -> Option<CoinType> {
        if self.coin_types.contains_key(&index) {
            return None;
        }
        let mut child_key = self.private_key.derive_child_key(index, true).unwrap(); // use hardened derivation
        child_key.testnet = index == BITCOIN_TESTNET_INDEX;
        println!("index: {}", index);
        println!("child_key.testnet: {}", child_key.testnet);
        self.coin_types
            .insert(index, CoinType::create(index, &child_key))
    }
}

impl CoinType {
    /// Creates a new coin type.
    fn create(index: u32, private_key: &ExtendedPrivateKey) -> CoinType {
        let mut private_key = private_key.clone();
        private_key.testnet = index == BITCOIN_TESTNET_INDEX;
        CoinType {
            index,
            name: COIN_TYPE_NAMES.get(&index).unwrap().to_string(),
            private_key,
            accounts: BTreeMap::new(),
        }
    }

    /// Adds a new account.
    fn add_account(&mut self) -> Account {
        let index = if self.accounts.len() == 0 {
            0
        } else {
            self.accounts.iter().max_by(|a, b| a.0.cmp(&b.0)).unwrap().0 + 1
        };
        let child_key = self.private_key.derive_child_key(index, true).unwrap(); // use hardened derivation
        let account = Account::create(index, &child_key);
        self.accounts.insert(index, account.clone());
        account
    }
}

impl Account {
    /// Creates a new account.
    fn create(index: u32, private_key: &ExtendedPrivateKey) -> Account {
        let external_chain_key = private_key.derive_child_key(0, false).unwrap();
        let internal_chain_key = private_key.derive_child_key(1, false).unwrap();
        let external_chain = Change::create(0, external_chain_key);
        let internal_chain = Change::create(1, internal_chain_key);
        Account {
            index,
            private_key: private_key.clone(),
            external_chain,
            internal_chain,
        }
    }

    /// Creates and returns a new keypair for return transaction change.
    fn new_change_keypair(&mut self, index: Option<u32>) -> Keypair {
        self.internal_chain.new_keypair(index)
    }

    /// Creates and returns a new keypair for receiving payments.
    fn new_receive_keypair(&mut self, index: Option<u32>) -> Keypair {
        self.external_chain.new_keypair(index)
    }
}

impl Change {
    /// Creates a new change type.
    fn create(index: u32, private_key: ExtendedPrivateKey) -> Change {
        assert!(index == 0 || index == 1);
        Change {
            index,
            private_key: private_key.clone(),
            keys: BTreeMap::new(),
        }
    }

    /// Creates and returns a new keypair.
    fn new_keypair(&mut self, index: Option<u32>) -> Keypair {
        let index = match index {
            Some(index) => index,
            None => {
                if self.keys.len() == 0 {
                    0
                } else {
                    self.keys.iter().max_by(|a, b| a.0.cmp(&b.0)).unwrap().0 + 1
                }
            }
        };
        let child_private_key = self.private_key.derive_child_key(index, false).unwrap();
        let child_public_key = child_private_key.derive_public_key();
        let keypair = Keypair::create(child_private_key, child_public_key);
        self.keys.insert(index, keypair.clone());
        keypair
    }
}

impl Keypair {
    /// Creates a new keypair.
    fn create(private_key: ExtendedPrivateKey, public_key: ExtendedPublicKey) -> Keypair {
        Keypair {
            private_key,
            public_key,
        }
    }
}
