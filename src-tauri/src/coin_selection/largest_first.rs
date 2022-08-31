//! Implements the Largest-First coin selection algorithm,
//! as specified in [CIP-2](https://cips.cardano.org/cips/cip2/).
use super::error::CoinSelectionError;
use super::fee_estimation;
use super::CoinSelection;
use super::MAX_INPUTS_PER_TX;
use crate::transactions::utxo::UTXOBox;

/// Returns a selection of UTXOs according to the Largest-First coin
/// selection algorithm.
pub fn select_coins(
    mut boxed_utxos: Vec<UTXOBox>,
    num_outputs: u32,
    target: u64,
    fee_per_byte: u64,
) -> Result<CoinSelection, CoinSelectionError> {
    // sort UTXOs descendingly with respect to the value
    boxed_utxos.sort_by_key(|utxo_box| -(utxo_box.utxo.value as i128));

    let mut selected_utxos = Vec::new();
    let mut selected_amount = 0;
    while !boxed_utxos.is_empty() {
        let utxo = boxed_utxos.remove(0);
        selected_amount += utxo.utxo.value;
        selected_utxos.push(utxo);

        if selected_utxos.len() > MAX_INPUTS_PER_TX {
            return Err(CoinSelectionError::new(
                "max_input_count_exceeded".to_string(),
            ));
        }
        // add output for potential change output (overestimate fee)
        let fee = fee_estimation::estimate_fee(
            selected_utxos.len() as u32,
            num_outputs + 1,
            fee_per_byte,
        );
        let target_plus_fee = target + fee;
        if selected_amount < target_plus_fee {
            continue;
        } else if selected_amount > target_plus_fee {
            let change = Some(selected_amount - target_plus_fee);
            return Ok(CoinSelection {
                selected_utxos,
                change,
            });
        } else {
            return Ok(CoinSelection {
                selected_utxos,
                change: None,
            });
        }
    }
    Err(CoinSelectionError::new("balance_insufficient".to_string()))
}
