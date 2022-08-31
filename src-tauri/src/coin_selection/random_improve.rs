//! Implements the Random-Improve coin selection algorithm,
//! as specified in [CIP-2](https://cips.cardano.org/cips/cip2/).
use super::error::CoinSelectionError;
use super::fee_estimation;
use super::largest_first;
use super::CoinSelection;
use super::MAX_INPUTS_PER_TX;
use crate::transactions::utxo::UTXOBox;
use rand::Rng;
use std::collections::HashMap;

/// Returns a selection of UTXOs according to the Random-Improve coin
/// selection algorithm.
pub fn select_coins(
    mut boxed_utxos: Vec<UTXOBox>,
    num_outputs: u32,
    target: u64,
    fee_per_byte: u64,
) -> Result<CoinSelection, CoinSelectionError> {
    // Phase 1: Random selection
    let mut selected_utxos = Vec::new();
    let mut selected_amount = 0;
    let mut target_plus_fee = target;
    while !boxed_utxos.is_empty() {
        if selected_utxos.len() > MAX_INPUTS_PER_TX {
            return largest_first::select_coins(boxed_utxos, num_outputs, target, fee_per_byte);
        }
        let index: usize = rand::thread_rng().gen_range(0..boxed_utxos.len());
        let utxo = boxed_utxos.remove(index);
        selected_amount += utxo.utxo.value;
        selected_utxos.push(utxo);
        // add output for potential change output (overestimate fee)
        let fee = fee_estimation::estimate_fee(
            selected_utxos.len() as u32,
            num_outputs + 1,
            fee_per_byte,
        );
        target_plus_fee = target + fee;
        if selected_amount >= target_plus_fee {
            break;
        }
    }
    if selected_amount < target_plus_fee {
        return Err(CoinSelectionError::new("Balance insufficient".to_string()));
    }

    // Phase 2: Improvement
    let mut boxed_utxos_map: HashMap<usize, UTXOBox> =
        boxed_utxos.into_iter().enumerate().collect();
    let mut indices = Vec::from_iter(0..boxed_utxos_map.len());
    let mut target_plus_fee = target_plus_fee as i128;
    while !indices.is_empty() {
        let index: usize = rand::thread_rng().gen_range(0..indices.len());
        let map_index = indices.get(index).unwrap();
        let utxo = boxed_utxos_map.get(map_index).unwrap();

        let fee = fee_estimation::estimate_fee(
            (selected_utxos.len() + 1) as u32,
            num_outputs + 1,
            fee_per_byte,
        );
        target_plus_fee = (target + fee) as i128;
        let ideal = target_plus_fee * 2;
        let maximum = target_plus_fee * 3;

        let new_amount = (selected_amount + utxo.utxo.value) as i128;
        let condition1 = (ideal - new_amount).abs() < (ideal - selected_amount as i128).abs();
        let condition2 = new_amount <= maximum;
        let condition3 = selected_utxos.len() + 1 <= MAX_INPUTS_PER_TX;

        if condition1 && condition2 && condition3 {
            let utxo = boxed_utxos_map.remove(map_index).unwrap();
            selected_amount += utxo.utxo.value;
            selected_utxos.push(utxo);
        }
        indices.remove(index);
    }
    if selected_amount > target_plus_fee as u64 {
        let change = Some(selected_amount - target_plus_fee as u64);
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

#[cfg(test)]
mod tests {
    use crate::coin_selection::random_improve;
    use crate::keys::{bip32::ExtendedPrivateKey, bip44::private_hierarchy::Keypair};
    use crate::transactions::transaction::TransactionOutput;
    use crate::transactions::utxo::{UTXOBox, UTXOStatus, UTXO};
    use num_bigint::BigUint;
    use std::str::FromStr;

    #[test]
    fn test_random_improve() {
        let balance = 90778;
        let target = 10000;
        let utxo_status = UTXOStatus {
            confirmed: true,
            block_height: None,
            block_hash: None,
            block_time: None,
        };
        let utxo = UTXO {
            txid: String::from("d8cb1a81c683dde549e474566345c4d74f649e6dad642aab7d5fcee5d4583e5a"),
            vout: 0,
            value: balance,
            status: utxo_status,
        };
        let output = TransactionOutput {
            scriptpubkey: String::from("76a9146bd18c889da9d66610354ccdc4676f055bae298088ac"),
            scriptpubkey_asm: String::from("OP_DUP OP_HASH160 OP_PUSHBYTES_20 6bd18c889da9d66610354ccdc4676f055bae2980 OP_EQUALVERIFY OP_CHECKSIG"),
            scriptpubkey_type: String::from("p2pkh"),
            scriptpubkey_address: String::from("mqM3dJApCknasvUkPEnALkVBCDjsFLmWQ1"),
            value: target,
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
        let selected_coins = random_improve::select_coins(vec![utxo_box], 1, target, 8).unwrap();
        assert!(selected_coins.change.unwrap() < balance);
        assert!(selected_coins.change.unwrap() + target <= balance);
    }
}
