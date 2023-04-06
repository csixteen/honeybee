use std::fmt::{self, Formatter};

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::errors::Error;

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
pub enum Unit {
    #[default]
    Auto,
    B,
    KiB,
    MiB,
    GiB,
    TiB,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            Unit::Auto => "auto",
            Unit::B => "B",
            Unit::KiB => "KiB",
            Unit::MiB => "MiB",
            Unit::GiB => "GiB",
            Unit::TiB => "TiB",
        })
    }
}

impl Unit {
    pub fn from_bytes(bytes: u64, target: Unit, decimals: usize) -> f64 {
        let d: f64 = match target {
            Unit::KiB => 1024_f64,
            Unit::MiB => 1048576_f64,
            Unit::GiB => 1073741824_f64,
            Unit::TiB => 1099511627776_f64,
            _ => 1_f64,
        };

        (bytes as f64 / d).truncate(decimals)
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
        match value {
            "auto" => Ok(Unit::Auto),
            _ => Unit::try_from(value.chars().nth(0).unwrap()),
        }
    }
}

pub trait Truncate<T> {
    fn truncate(self, n: usize) -> T;
}

impl Truncate<f32> for f32 {
    fn truncate(self, n: usize) -> f32 {
        let x = 10.0_f32.powf(n as f32);
        f32::trunc(self * x) / x
    }
}

impl Truncate<f64> for f64 {
    fn truncate(self, n: usize) -> f64 {
        let x = 10.0_f64.powf(n as f64);
        f64::trunc(self * x) / x
    }
}
