//! Implements the coin selection strategy.
pub mod error;
pub mod fee_estimation;
pub mod largest_first;
pub mod random_improve;
use crate::transactions::utxo::UTXOBox;
use serde::{Deserialize, Serialize};

static MAX_INPUTS_PER_TX: usize = 2048; // this value is not set in stone

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinSelection {
    pub selected_utxos: Vec<UTXOBox>,
    pub change: Option<u64>,
}
