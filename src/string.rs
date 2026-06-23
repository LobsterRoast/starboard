use anyhow::{anyhow, bail};

// This represents a string with a maximum capacity that can be stored without a heap allocation
#[derive(Debug, Copy, Clone)]
pub struct StarboardString {
    inner: [char; 30],
    count: usize,
}

impl TryFrom<String> for StarboardString {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, anyhow::Error> {
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
