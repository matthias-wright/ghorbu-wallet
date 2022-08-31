//! Estimates the value of the transaction fee based
//! on the transaction size in bytes and the current
//! recommended fee per byte.
use crate::utils::varint;

static TX_INPUT_SIZE: u32 = 147; // in bytes
static TX_OUTPUT_SIZE: u32 = 34; // in bytes

/// Returns the estimated fee of the transaction.
pub fn estimate_fee(num_inputs: u32, num_outputs: u32, fee_per_byte: u64) -> u64 {
    (estimate_transaction_size(num_inputs, num_outputs) as u64) * fee_per_byte
}

/// Returns the estimated size of the transaction in bytes.
pub fn estimate_transaction_size(num_inputs: u32, num_outputs: u32) -> u32 {
    let mut size = 4; // version
                      // the cast from usize (vector length) to u32 cannot overflow
                      // because the vector will never be longer than 9
    size += varint::encode(num_inputs as u64).len() as u32;
    size += num_inputs * TX_INPUT_SIZE;
    size += varint::encode(num_outputs as u64).len() as u32;
    size += num_outputs * TX_OUTPUT_SIZE;
    size += 4; // locktime
    size
}
