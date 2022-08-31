//! Defines errors for the coin selection module.
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when a transaction cannot
/// be constructed from the available UTXOs.
pub struct CoinSelectionError {
    message: String,
}

impl CoinSelectionError {
    pub fn new(message: String) -> CoinSelectionError {
        CoinSelectionError { message }
    }
}

impl fmt::Display for CoinSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for CoinSelectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CoinSelectionError {}
