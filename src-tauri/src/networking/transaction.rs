//! Implements the networking functionality for transactions.
//! The [mempool.space API](https://mempool.space/docs/api/rest) is used
//! to interact with the blockchain.
use super::{error::SendTransactionError, BITCOIN_API, BITCOIN_TESTNET_API};
use crate::keys::address::{Address, SimpleAddress};
use crate::transactions::transaction::{SimplifiedTransaction, Transaction, TransactionType};
use serde_json;
use std::cmp::Ordering;
use std::collections::HashSet;

/// Returns the transaction for the specified transaction ID.
pub async fn get_transaction(txid: &str, testnet: bool) -> Result<Transaction, String> {
    let api_url = if testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    match reqwest::get(&format!("{}/tx/{}", api_url, txid)).await {
        Ok(resp) => match resp.text().await {
            Ok(resp) => match serde_json::from_str(&resp) {
                Ok(tx) => Ok(tx),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err.to_string()),
    }
}

/// Returns all transactions for the specified address.
pub async fn get_address_transactions(
    address: &Address,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let api_url = if address.testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    let resp = reqwest::get(&format!("{}/address/{}/txs", api_url, address.to_string()))
        .await?
        .text()
        .await?;
    let transactions: Vec<Transaction> = serde_json::from_str(&resp)?;
    Ok(transactions)
}

/// Returns all transactions for the specified addresses.
pub async fn get_addresses_transactions(
    addresses: Vec<Address>,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let mut txs = Vec::new();
    for address in addresses {
        txs.extend(get_address_transactions(&address).await?);
    }
    Ok(txs)
}

/// Returns all transactions for the specified addresses.
pub async fn get_addresses_simple_transactions(
    addresses: Vec<Address>,
) -> Result<Vec<SimplifiedTransaction>, Box<dyn std::error::Error>> {
    let address_set =
        HashSet::<String>::from_iter(addresses.iter().map(|address| address.to_string()));
    let mut txs = get_addresses_transactions(addresses).await?;
    txs.sort_by(|a, b| {
        let a_status = a.status.clone().unwrap();
        let b_status = b.status.clone().unwrap();
        if !a_status.confirmed {
            Ordering::Less
        } else if !b_status.confirmed {
            Ordering::Greater
        } else {
            if a_status.block_height.unwrap() >= b_status.block_height.unwrap() {
                // transactions with higher block height should come first
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    });

    let mut txs_set = HashSet::new();
    let mut simple_txs = Vec::new();
    for tx in txs {
        if txs_set.contains(&tx.txid) {
            continue;
        }
        txs_set.insert(tx.txid);
        let mut incoming = true;
        for txin in tx.vin {
            if address_set.contains(&txin.prevout.scriptpubkey_address) {
                incoming = false;
                break;
            }
        }
        let mut send_amount = 0;
        let mut received_amount = 0;
        for txout in tx.vout {
            if address_set.contains(&txout.scriptpubkey_address) {
                // transaction output is send to wallet
                received_amount += txout.value;
            } else {
                send_amount += txout.value;
            }
        }
        let (transaction_type, value) = if !incoming && send_amount == 0 {
            (TransactionType::Internal, 0)
        } else if incoming {
            (TransactionType::Incoming, received_amount)
        } else {
            (TransactionType::Outgoing, send_amount)
        };
        simple_txs.push(SimplifiedTransaction {
            transaction_type,
            value,
            fee: tx.fee.unwrap(),
            confirmed: tx.status.unwrap().confirmed,
        });
    }
    Ok(simple_txs)
}

/// Marks addresses as used if transactions exist.
pub async fn mark_addresses_as_used(
    addresses: Vec<Address>,
) -> Result<Vec<SimpleAddress>, Box<dyn std::error::Error>> {
    let mut simple_addresses = Vec::new();
    for address in addresses {
        let txs = get_address_transactions(&address).await?;
        simple_addresses.push(SimpleAddress {
            address: address.to_string(),
            used: !txs.is_empty(),
        });
    }
    Ok(simple_addresses)
}

/// Send a raw transaction.
pub async fn send_transaction(
    tx: Transaction,
    testnet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_url = if testnet {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    let client = reqwest::Client::new();
    let res = client
        .post(&format!("{}/tx", api_url))
        .body(tx.serialize_hex())
        .send()
        .await?;
    let status = res.text().await?;
    if status.contains("error") {
        Err(Box::new(SendTransactionError {}))
    } else {
        Ok(())
    }
}
