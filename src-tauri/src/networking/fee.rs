//! Implements the networking functionality for fees.
//! The [mempool.space API](https://mempool.space/docs/api/rest) is used
//! to interact with the blockchain.
use super::{BITCOIN_API, BITCOIN_TESTNET_API};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub fastest_fee: u32,
    pub half_hour_fee: u32,
    pub hour_fee: u32,
    pub economy_fee: u32,
    pub minimum_fee: u32,
}

/// Returns the current recommended fees.
pub async fn get_recommended_fees(
    coin_type_index: u32,
) -> Result<Fees, Box<dyn std::error::Error>> {
    let api_url = if coin_type_index == 1 {
        BITCOIN_TESTNET_API
    } else {
        BITCOIN_API
    };
    let resp = reqwest::get(&format!("{}/v1/fees/recommended", api_url))
        .await?
        .text()
        .await?;
    let fees: Fees = serde_json::from_str(&resp)?;
    Ok(fees)
}
