use serde::Deserialize;
use std::{num::NonZeroI32, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Limit(u8);

impl Limit {
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 10 { Some(Self(value)) } else { None }
    }
    pub fn get(self) -> u8 { self.0 }
}
impl FromStr for Limit {
    type Err = std::num::ParseIntError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> { raw.parse().map(Self) }
}

pub fn decode(raw: &str) -> Result<Limit, serde_json::Error> {
    serde_json::from_str(raw)
}

pub fn negate(value: NonZeroI32) -> Option<NonZeroI32> {
    NonZeroI32::new(-value.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_values() {
        assert_eq!(decode("3").unwrap().get(), 3);
        assert_eq!(negate(NonZeroI32::new(2).unwrap()).unwrap().get(), -2);
    }
}
