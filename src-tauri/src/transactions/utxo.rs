use crate::keys::bip44::Keypair;
use crate::transactions::transaction::TransactionOutput;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UTXO {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub status: UTXOStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UTXOStatus {
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

/// Store the UTXO together with the corresponding key pair and the transaction output
/// to enable coin selection and signing.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UTXOBox {
    pub utxo: UTXO,
    pub output: TransactionOutput,
    pub keypair: Keypair,
}
