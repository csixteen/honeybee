use std::fmt::{self, Formatter};

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::errors::*;

const MAX_EXPONENT: usize = 4;

#[derive(Clone, Copy, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
pub enum Unit {
    B,
    KiB,
    MiB,
    #[default]
    GiB,
    TiB,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            Unit::B => "B",
            Unit::KiB => "KiB",
            Unit::MiB => "MiB",
            Unit::GiB => "GiB",
            Unit::TiB => "TiB",
        })
    }
}

impl Unit {
    pub fn from_bytes(bytes: u64, target: Unit) -> (f64, Unit) {
        let mut base = bytes as f64;
        let mut exponent = 0_usize;

        while base >= 1024_f64 && exponent < MAX_EXPONENT {
            if target == Unit::try_from(exponent).unwrap() {
                break;
            }

            base /= 1024_f64;
            exponent += 1;
        }

        (base, Unit::try_from(exponent).unwrap())
    }

    pub fn convert_to_bytes(amount: u64, unit: Unit) -> u64 {
        amount
            * (match unit {
                Self::KiB => 1024,
                Self::MiB => 1048576,
                Self::GiB => 1073741824,
                Self::TiB => 1099511627776,
                _ => 1,
            })
    }
}

impl TryFrom<usize> for Unit {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::B),
            1 => Ok(Self::KiB),
            2 => Ok(Self::MiB),
            3 => Ok(Self::GiB),
            4 => Ok(Self::TiB),
            _ => Err(Error::new("Invalid Unit discriminant")),
        }
    }
}

impl TryFrom<char> for Unit {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'b' | 'B' => Ok(Self::B),
            'k' | 'K' => Ok(Self::KiB),
            'm' | 'M' => Ok(Self::MiB),
            'g' | 'G' => Ok(Self::GiB),
            't' | 'T' => Ok(Self::TiB),
            _ => Err(Error::new("Unknown unit")),
        }
    }
}

impl TryFrom<&str> for Unit {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Unit::try_from(value.chars().next().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        assert_eq!((512_f64, Unit::MiB), Unit::from_bytes(536870912, Unit::GiB));
        assert_eq!((1_f64, Unit::GiB), Unit::from_bytes(1073741824, Unit::GiB));
    }
}
