//! Defines errors for the networking module.
use std::error::Error as StdError;
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when sending a
/// transaction fails.
pub struct SendTransactionError;

impl fmt::Display for SendTransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SendTransactionError")
    }
}

impl fmt::Debug for SendTransactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SendTransactionError")
    }
}

impl StdError for SendTransactionError {}
