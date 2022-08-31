//! Defines errors for the key module.
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when creating a Keystore fails.
pub struct CreateKeystoreError;

impl fmt::Display for CreateKeystoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CreateKeystoreError")
    }
}

impl fmt::Debug for CreateKeystoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CreateKeystoreError")
    }
}

/// This error occurs when an imported key is corrupted.
pub struct ImportKeyError {
    message: String,
}

impl ImportKeyError {
    pub fn new(message: &str) -> ImportKeyError {
        ImportKeyError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ImportKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for ImportKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// This error occurs when a child key derivation fails.
pub struct ChildKeyDeriveError {
    message: String,
}

impl ChildKeyDeriveError {
    pub fn new(message: &str) -> ChildKeyDeriveError {
        ChildKeyDeriveError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ChildKeyDeriveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for ChildKeyDeriveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// This error occurs when parsing a Base58Check address fails.
pub struct ParseAddressError {
    message: String,
}

impl ParseAddressError {
    pub fn new(message: &str) -> ParseAddressError {
        ParseAddressError {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for ParseAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
