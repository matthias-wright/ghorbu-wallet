//! Implements the networking functionality for UTXOs.
//! The [mempool.space API](https://mempool.space/docs/api/rest) is used
//! to interact with the blockchain.
use super::transaction;
use super::{BITCOIN_API, BITCOIN_TESTNET_API};
use crate::keys::address::Address;
use crate::keys::bip44::Keypair;
use crate::transactions::utxo::{UTXOBox, UTXO};
use serde_json;

/// Returns all UTXOs for the specified address.
pub async fn _get_address_utxos(
    address: &Address,
) -> Result<Vec<UTXO>, Box<dyn std::error::Error>> {
    let api_url = if address.testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    let resp = reqwest::get(&format!("{}/address/{}/utxo", api_url, address.to_string()))
        .await?
        .text()
        .await?;
    let utxos: Vec<UTXO> = serde_json::from_str(&resp)?;
    Ok(utxos)
}

/// Returns all UTXOs for the specified address.
pub async fn get_address_utxos(address: &Address) -> Result<Vec<UTXO>, String> {
    let api_url = if address.testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    match reqwest::get(&format!("{}/address/{}/utxo", api_url, address.to_string())).await {
        Ok(resp) => match resp.text().await {
            Ok(resp) => match serde_json::from_str(&resp) {
                Ok(utxo) => Ok(utxo),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err.to_string()),
    }
}

/// Returns all UTXOs for the specified address.
pub fn get_address_utxos_blocking(
    address: &Address,
) -> Result<Vec<UTXO>, Box<dyn std::error::Error>> {
    let api_url = if address.testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    let resp =
        reqwest::blocking::get(&format!("{}/address/{}/utxo", api_url, address.to_string()))?
            .text()?;
    let utxos: Vec<UTXO> = serde_json::from_str(&resp)?;
    Ok(utxos)
}

/// Returns all UTXOs for the specified addresses.
pub async fn get_addresses_utxos(
    addresses: Vec<Address>,
) -> Result<Vec<UTXO>, Box<dyn std::error::Error>> {
    let mut utxos = Vec::new();
    for address in addresses {
        utxos.extend(get_address_utxos(&address).await?);
    }
    Ok(utxos)
}

/// Returns the balance for the specified addresses.
pub async fn get_account_balance(
    addresses: Vec<Address>,
) -> Result<u64, Box<dyn std::error::Error>> {
    let balance = get_addresses_utxos(addresses)
        .await?
        .iter()
        .map(|utxo| utxo.value)
        .sum();
    Ok(balance)
}

/// Returns all boxed UTXOs for the specified key pair.
pub async fn get_keypair_boxed_utxos(keypair: &Keypair) -> Result<Vec<UTXOBox>, String> {
    let mut boxed_utxos = Vec::new();
    match get_address_utxos(&keypair.public_key.get_address()).await {
        Ok(utxos) => {
            for utxo in utxos {
                let tx =
                    transaction::get_transaction(&utxo.txid, keypair.public_key.testnet).await?;
                match tx.vout.get(utxo.vout as usize) {
                    Some(output) => boxed_utxos.push(UTXOBox {
                        utxo,
                        output: output.clone(),
                        keypair: keypair.clone(),
                    }),
                    None => continue, // invalid index, ignore UTXO
                }
            }
            Ok(boxed_utxos)
        }
        Err(err) => Err(err.to_string()),
    }
}

/// Returns all boxed UTXOs for the specified key pairs.
pub async fn get_keypairs_boxed_utxos(keypairs: Vec<Keypair>) -> Result<Vec<UTXOBox>, String> {
    let mut boxed_utxos = Vec::new();
    for keypair in keypairs {
        boxed_utxos.extend(get_keypair_boxed_utxos(&keypair).await?);
    }
    Ok(boxed_utxos)
}
