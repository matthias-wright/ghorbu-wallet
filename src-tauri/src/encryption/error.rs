//! Defines errors for the encryption module.
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when the decryption fails.
pub struct WrongPasswordError;

impl fmt::Display for WrongPasswordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WrongPasswordError")
    }
}

impl fmt::Debug for WrongPasswordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WrongPasswordError")
    }
}

impl Error for WrongPasswordError {}
