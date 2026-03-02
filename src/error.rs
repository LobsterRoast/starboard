use std::fmt::{self, Result};

#[derive(Debug)]
pub struct StarboardError {
    info: &'static str,
    code: u8,
}

impl fmt::Display for StarboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        write!(f, "Error Code {}: {}", &self.code, &self.info)
    }
}

impl std::error::Error for StarboardError {}

impl StarboardError {
    pub fn new(code: u8, info: &'static str) -> Self {
        Self { info, code }
    }

    pub fn info(&self) -> &'static str {
        &self.info
    }

    pub fn code(&self) -> &u8 {
        &self.code
    }
}
