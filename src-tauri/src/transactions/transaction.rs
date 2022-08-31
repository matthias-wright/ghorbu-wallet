use super::{
    error::UnsupportedScriptError,
    script::{self, ScriptType},
    utxo::UTXOBox,
};
use crate::{
    keys::address::Address,
    utils::{hex, varint},
};
use bitcoin_hashes::{sha256, Hash};
use secp256k1::{Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};

static SIGHASH_ALL: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub txid: String,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<TransactionInput>,
    pub vout: Vec<TransactionOutput>,
    pub size: Option<u32>,
    pub weight: Option<u32>,
    pub fee: Option<u64>,
    pub status: Option<TransactionStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionStatus {
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionInput {
    pub txid: String,
    pub vout: u32,
    pub prevout: TransactionOutput,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionOutput {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: String,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimplifiedTransaction {
    pub transaction_type: TransactionType,
    pub value: u64,
    pub fee: u64,
    pub confirmed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionType {
    Incoming,
    Outgoing,
    Internal,
}

impl Transaction {
    /// Creates a transaction.
    pub fn create(
        boxed_utxos: &Vec<UTXOBox>,
        targets: Vec<(Address, u64)>,
    ) -> Result<Transaction, UnsupportedScriptError> {
        let tx_ins: Result<Vec<TransactionInput>, UnsupportedScriptError> = boxed_utxos
            .iter()
            .map(|utxo| TransactionInput::create(&utxo))
            .collect();
        let tx_outs: Vec<TransactionOutput> = targets
            .iter()
            .map(|(address, amount)| TransactionOutput::create(address, *amount, ScriptType::P2PKH))
            .collect();

        Ok(Transaction {
            txid: String::from(""),
            version: 1,
            locktime: 0,
            vin: tx_ins?,
            vout: tx_outs,
            size: None,
            weight: None,
            fee: None,
            status: None,
        })
    }

    /// Signs all transaction inputs.
    pub fn sign_all_inputs(&mut self, boxed_utxos: &Vec<UTXOBox>) {
        boxed_utxos
            .iter()
            .enumerate()
            .for_each(|(index, utxo)| self.sign_input(index, utxo));
    }

    /// Signs the transaction input with the specified index.
    pub fn sign_input(&mut self, input_index: usize, utxo_box: &UTXOBox) {
        let z = self.signature_hash(input_index, &utxo_box.output);
        let msg = Message::from_slice(&z).unwrap();
        let secp = Secp256k1::new();
        let private_key = SecretKey::from_slice(&utxo_box.keypair.private_key.key_data).unwrap();
        let mut sig_der = secp.sign_ecdsa(&msg, &private_key).serialize_der().to_vec();
        sig_der.extend(&SIGHASH_ALL.to_be_bytes()[3..]);
        let pubkey_sec = &utxo_box.keypair.public_key.key_data;
        let script_sig = script::p2pkh_script_sig(
            &hex::bytes_to_hex(&sig_der),
            &hex::bytes_to_hex(&pubkey_sec),
        );
        let tx_in = self.vin.get_mut(input_index).unwrap();
        tx_in.scriptsig_asm = script_sig;
    }

    /// Returns the signature hash for signing the input with the specified index.
    pub fn signature_hash(&mut self, input_index: usize, output: &TransactionOutput) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes()[..4]);
        bytes.extend(varint::encode(self.vin.len() as u64));
        self.vin.iter().enumerate().for_each(|(index, tx_in)| {
            let mut tx_in_clone = tx_in.clone();
            if index == input_index {
                tx_in_clone.scriptsig_asm = output.scriptpubkey_asm.clone();
            } else {
                tx_in_clone.scriptsig_asm = String::from("");
            }
            bytes.extend(&tx_in_clone.serialize());
        });
        bytes.extend(varint::encode(self.vout.len() as u64));
        self.vout
            .iter()
            .for_each(|tx_out| bytes.extend(&tx_out.serialize()));
        bytes.extend(&self.locktime.to_le_bytes()[..4]);
        bytes.extend(&SIGHASH_ALL.to_le_bytes()[..4]);
        let hash = sha256::Hash::hash(&bytes);
        sha256::Hash::hash(&hash).into_inner()
    }

    /// Returns the serialized transaction.
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes()[..4]);
        bytes.extend(varint::encode(self.vin.len() as u64));
        self.vin
            .iter()
            .for_each(|tx_in| bytes.extend(&tx_in.serialize()));
        bytes.extend(varint::encode(self.vout.len() as u64));
        self.vout
            .iter()
            .for_each(|tx_out| bytes.extend(&tx_out.serialize()));
        bytes.extend(&self.locktime.to_le_bytes()[..4]);
        bytes
    }

    /// Returns the serialized transaction in hex format.
    pub fn serialize_hex(&self) -> String {
        hex::bytes_to_hex(&self.serialize())
    }
}

impl TransactionInput {
    /// Creates a transaction input.
    pub fn create(utxo_box: &UTXOBox) -> Result<TransactionInput, UnsupportedScriptError> {
        if utxo_box.output.scriptpubkey_type == "p2pkh" {
            Ok(TransactionInput {
                txid: utxo_box.utxo.txid.clone(),
                vout: utxo_box.utxo.vout,
                prevout: utxo_box.output.clone(),
                scriptsig: String::from(""),
                scriptsig_asm: String::from(""),
                witness: None,
                is_coinbase: false,
                sequence: 0xffffffff,
            })
        } else {
            Err(UnsupportedScriptError::new(
                utxo_box.output.scriptpubkey_type.clone(),
            ))
        }
    }

    /// Returns the serialized transaction input.
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let txid_le_bytes: Vec<u8> = hex::hex_to_bytes(&self.txid)
            .unwrap()
            .into_iter()
            .rev()
            .collect();
        bytes.extend(txid_le_bytes);
        bytes.extend(&self.vout.to_le_bytes()[..4]);
        bytes.extend(script::serialize(&self.scriptsig_asm).unwrap());
        bytes.extend(&self.sequence.to_le_bytes()[..4]);
        bytes
    }
}

impl TransactionOutput {
    /// Creates a transaction output.
    pub fn create(
        target_address: &Address,
        amount: u64,
        scriptpubkey_type: ScriptType,
    ) -> TransactionOutput {
        match scriptpubkey_type {
            ScriptType::P2PKH => TransactionOutput {
                scriptpubkey: String::from(""),
                scriptpubkey_asm: script::p2pkh_script_pub_key(&target_address.get_h160()),
                scriptpubkey_type: scriptpubkey_type.to_string(),
                scriptpubkey_address: target_address.to_string(),
                value: amount,
            },
        }
    }

    /// Returns the serialized transaction output.
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.value.to_le_bytes()[..8]);
        bytes.extend(script::serialize(&self.scriptpubkey_asm).unwrap());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::{TransactionInput, TransactionOutput};
    use crate::{
        keys::{bip32::ExtendedPrivateKey, bip44::private_hierarchy::Keypair},
        transactions::{
            transaction::Transaction,
            utxo::{UTXOBox, UTXOStatus, UTXO},
        },
        utils::hex,
    };
    use num_bigint::BigUint;
    use std::str::FromStr;

    #[test]
    fn test_tx_in_serialize() {
        let tx_out = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from(""),
            scriptpubkey_type: String::from(""),
            scriptpubkey_address: String::from(""),
            value: 0,
        };
        let tx_in = TransactionInput {
            txid: String::from("ce5f6f81800095fb6d054763cd352c3d64508b3632c01a2e9b71dce7e6ab3bd6"),
            vout: 1,
            prevout: tx_out,
            scriptsig: String::from(""),
            scriptsig_asm: String::from(""),
            witness: None,
            is_coinbase: false,
            sequence: 0xffffffff,
        };
        let target = String::from(
            "d63babe6e7dc719b2e1ac032368b50643d2c35cd6347056dfb950080816f5fce0100000000ffffffff",
        );
        assert_eq!(hex::bytes_to_hex(&tx_in.serialize()), target);
    }

    #[test]
    fn test_tx_out_serialize() {
        let tx_out = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 fc20f7fc8b0a6785e02ebe93adbcc66f3065c997 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from(""),
            value: 1000,
        };
        let target =
            String::from("e8030000000000001976a914fc20f7fc8b0a6785e02ebe93adbcc66f3065c99788ac");
        assert_eq!(hex::bytes_to_hex(&tx_out.serialize()), target);
    }

    #[test]
    fn test_tx_serialize() {
        let tx_out = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from(""),
            scriptpubkey_type: String::from(""),
            scriptpubkey_address: String::from(""),
            value: 0,
        };
        let tx_in = TransactionInput {
            txid: String::from("5c72fb2038f71b83662b5625178bf723d571d08f022bc33c33ed40ddb9234965"),
            vout: 0,
            prevout: tx_out,
            scriptsig: String::from(""),
            scriptsig_asm: String::from(""),
            witness: None,
            is_coinbase: false,
            sequence: 0xffffffff,
        };
        let tx_out1 = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 fc20f7fc8b0a6785e02ebe93adbcc66f3065c997 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from(""),
            value: 1000,
        };
        let tx_out2 = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 6bd18c889da9d66610354ccdc4676f055bae2980 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from(""),
            value: 7000,
        };
        let tx = Transaction {
            txid: String::from(""),
            version: 1,
            locktime: 0,
            vin: vec![tx_in],
            vout: vec![tx_out1, tx_out2],
            size: None,
            weight: None,
            fee: None,
            status: None,
        };
        let target = String::from("0100000001654923b9dd40ed333cc32b028fd071d523f78b1725562b66831bf73820fb725c0000000000ffffffff\
                                   02e8030000000000001976a914fc20f7fc8b0a6785e02ebe93adbcc66f3065c99788ac581b0000000000001976a91\
                                   46bd18c889da9d66610354ccdc4676f055bae298088ac00000000");
        assert_eq!(tx.serialize_hex(), target);
    }

    #[test]
    fn test_tx_sign_serialize() {
        let utxo_status = UTXOStatus {
            confirmed: true,
            block_height: None,
            block_hash: None,
            block_time: None,
        };
        let utxo = UTXO {
            txid: String::from("d8cb1a81c683dde549e474566345c4d74f649e6dad642aab7d5fcee5d4583e5a"),
            vout: 0,
            value: 10000,
            status: utxo_status,
        };
        let output = TransactionOutput {
            scriptpubkey: String::from("76a9146bd18c889da9d66610354ccdc4676f055bae298088ac"),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 OP_PUSHBYTES_20 6bd18c889da9d66610354ccdc4676f055bae2980 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from("mqM3dJApCknasvUkPEnALkVBCDjsFLmWQ1"),
            value: 10000,
        };
        let tx_in = TransactionInput {
            txid: String::from("d8cb1a81c683dde549e474566345c4d74f649e6dad642aab7d5fcee5d4583e5a"),
            vout: 0,
            prevout: output.clone(),
            scriptsig: String::from(""),
            scriptsig_asm: String::from(""),
            witness: None,
            is_coinbase: false,
            sequence: 0xffffffff,
        };
        let tx_out1 = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 fd158402792612f4d87a9f5f37e14a584e364a65 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from(""),
            value: 1000,
        };
        let tx_out2 = TransactionOutput {
            scriptpubkey: String::from(""),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 6bd18c889da9d66610354ccdc4676f055bae2980 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from(""),
            value: 8800,
        };
        let mut tx = Transaction {
            txid: String::from(""),
            version: 1,
            locktime: 0,
            vin: vec![tx_in],
            vout: vec![tx_out1, tx_out2],
            size: None,
            weight: None,
            fee: None,
            status: None,
        };
        let secret_num = BigUint::from_str(
            "54471658843786062176644521799104358682409094809685530415586086977504002449585",
        )
        .unwrap();
        let private_key = ExtendedPrivateKey {
            testnet: true,
            depth: 0x00,
            fingerprint: [0; 4],
            child_number: [0; 4],
            chain_code: [0; 32],
            key_data: secret_num.to_bytes_be().try_into().unwrap(),
        };
        let public_key = private_key.derive_public_key();
        let keypair = Keypair {
            private_key,
            public_key,
        };
        let utxo_box = UTXOBox {
            utxo,
            output,
            keypair,
        };
        tx.sign_input(0, &utxo_box);

        let target = String::from("01000000015a3e58d4e5ce5f7dab2a64ad6d9e644fd7c445635674e449e5dd83c6811acb\
                                   d8000000006b48304502210082d5afc04466b7566bcc44a4670980393edbfa88d0daf02c\
                                   163372fdcb5a1dc902203aa732322fd0cfca0d7fef4889779471d832dc0fa73ff5518a30\
                                   f92054b02d51012103597f57b176a4fd0bbf9b163ad341ed002101572b595485c537c367\
                                   3281a83ebcffffffff02e8030000000000001976a914fd158402792612f4d87a9f5f37e1\
                                   4a584e364a6588ac60220000000000001976a9146bd18c889da9d66610354ccdc4676f05\
                                   5bae298088ac00000000");
        assert_eq!(tx.serialize_hex(), target);
    }
}
