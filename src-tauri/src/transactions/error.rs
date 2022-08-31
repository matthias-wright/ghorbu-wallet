//! Defines errors for the transactions module.
use std::fmt;
use std::fmt::Formatter;

/// This error occurs when parsing a script type fails.
pub struct ParseScriptTypeError;

impl fmt::Display for ParseScriptTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseScriptTypeError")
    }
}

impl fmt::Debug for ParseScriptTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ParseScriptTypeError")
    }
}

/// This error occurs when a unsupported script type
/// is encountered.
pub struct UnsupportedScriptError {
    message: String,
}

impl UnsupportedScriptError {
    pub fn new(message: String) -> UnsupportedScriptError {
        UnsupportedScriptError { message }
    }
}

impl fmt::Display for UnsupportedScriptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for UnsupportedScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
