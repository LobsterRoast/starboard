use core::{
    fmt::{Display, Formatter},
    iter::IntoIterator,
};

use std::fmt;

use bincode::{Decode, Encode};

use anyhow::{anyhow, bail};

// This represents a string with a maximum capacity that can be stored without a heap allocation
#[derive(Debug, Copy, Clone, Decode, Encode)]
pub struct StarboardString {
    inner: [char; 30],
    count: usize,
}

impl TryFrom<String> for StarboardString {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut chars: Vec<char> = value.chars().collect();
        let count = chars.len();
        chars.resize_with(30, || '\0');
        Ok(Self {
            inner: chars
                .try_into()
                .map_err(|_| anyhow!("`StarboardString` must be 30 characters or less."))?,
            count,
        })
    }
}

impl Display for StarboardString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = String::from_iter(self.inner);
        write!(f, "{string}")
    }
}

impl Into<String> for StarboardString {
    fn into(self) -> String {
        String::from_iter(self.inner.into_iter().take(self.count))
    }
}

impl TryFrom<&str> for StarboardString {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}
