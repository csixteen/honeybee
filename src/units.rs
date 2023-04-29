use std::fmt::{self, Formatter};

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::errors::*;

const MAX_EXPONENT: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum Unit {
    Iec(IecSymbol),
    Si(SiSymbol),
}

impl Unit {
    pub fn iec_from_str(value: &str) -> Unit {
        Unit::Iec(IecSymbol::try_from(value).expect("Invalid IEC symbol"))
    }

    pub fn iec_from_char(c: char) -> Unit {
        Unit::Iec(IecSymbol::try_from(c).expect("Invalid IEC symbol"))
    }

    pub fn si_from_str(value: &str) -> Unit {
        Unit::Si(SiSymbol::try_from(value).expect("Invalid SI symbol"))
    }

    pub fn si_from_char(c: char) -> Unit {
        Unit::Si(SiSymbol::try_from(c).expect("Invalid SI symbol"))
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Iec(iec) => write!(f, "{}", iec),
            Unit::Si(si) => write!(f, "{}", si),
        }
    }
}

impl From<Unit> for u32 {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Iec(iec) => iec.into(),
            Unit::Si(si) => si.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
pub enum IecSymbol {
    None,
    KiB,
    MiB,
    #[default]
    GiB,
    TiB,
}

impl TryFrom<usize> for IecSymbol {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::KiB),
            2 => Ok(Self::MiB),
            3 => Ok(Self::GiB),
            4 => Ok(Self::TiB),
            _ => Err(Error::new("Invalid IEC symbol")),
        }
    }
}

impl From<IecSymbol> for u32 {
    fn from(value: IecSymbol) -> Self {
        value as u32
    }
}

impl fmt::Display for IecSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            IecSymbol::None => "B",
            IecSymbol::KiB => "KiB",
            IecSymbol::MiB => "MiB",
            IecSymbol::GiB => "GiB",
            IecSymbol::TiB => "TiB",
        })
    }
}

impl TryFrom<char> for IecSymbol {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'b' | 'B' => Ok(Self::None),
            'k' | 'K' => Ok(Self::KiB),
            'm' | 'M' => Ok(Self::MiB),
            'g' | 'G' => Ok(Self::GiB),
            't' | 'T' => Ok(Self::TiB),
            _ => Err(Error::new("Unknown IEC symbol")),
        }
    }
}

impl TryFrom<&str> for IecSymbol {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        IecSymbol::try_from(value.chars().next().unwrap())
    }
}

#[derive(Clone, Copy, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
pub enum SiSymbol {
    None,
    K,
    M,
    #[default]
    G,
    T,
}

impl fmt::Display for SiSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match &self {
            SiSymbol::None => "B",
            SiSymbol::K => "K",
            SiSymbol::M => "M",
            SiSymbol::G => "G",
            SiSymbol::T => "T",
        })
    }
}

impl TryFrom<usize> for SiSymbol {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::K),
            2 => Ok(Self::M),
            3 => Ok(Self::G),
            4 => Ok(Self::T),
            _ => Err(Error::new("Invalid SI symbol")),
        }
    }
}

impl From<SiSymbol> for u32 {
    fn from(value: SiSymbol) -> Self {
        value as u32
    }
}

impl TryFrom<char> for SiSymbol {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'b' | 'B' => Ok(Self::None),
            'k' | 'K' => Ok(Self::K),
            'm' | 'M' => Ok(Self::M),
            'g' | 'G' => Ok(Self::G),
            't' | 'T' => Ok(Self::T),
            _ => Err(Error::new("Unknown SI symbol")),
        }
    }
}

impl TryFrom<&str> for SiSymbol {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SiSymbol::try_from(value.chars().next().unwrap())
    }
}

pub fn bytes_to_unit(bytes: u64, target: Unit) -> (f64, Unit) {
    let mut b = bytes as f64;
    let mut exponent = 0_usize;
    let base = match target {
        Unit::Iec(_) => 1024_f64,
        Unit::Si(_) => 1000_f64,
    };

    while b >= base && exponent < MAX_EXPONENT {
        if u32::from(target) as usize == exponent {
            break;
        }

        b /= base;
        exponent += 1;
    }

    let new_symbol = match target {
        Unit::Iec(_) => Unit::Iec(IecSymbol::try_from(exponent).unwrap()),
        Unit::Si(_) => Unit::Si(SiSymbol::try_from(exponent).unwrap()),
    };

    (b, new_symbol)
}

pub fn unit_to_bytes(amount: u64, unit: Unit) -> u64 {
    amount
        * match unit {
            Unit::Iec(iec) => 1024_u64.pow(iec.into()),
            Unit::Si(si) => 1000_u64.pow(si.into()),
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_unit() {
        assert_eq!(
            bytes_to_unit(1024, Unit::Iec(IecSymbol::KiB)),
            (1_f64, Unit::Iec(IecSymbol::KiB))
        );
        assert_eq!(
            bytes_to_unit(1024 * 1024, Unit::Iec(IecSymbol::KiB)),
            (1024_f64, Unit::Iec(IecSymbol::KiB))
        );
        assert_eq!(
            bytes_to_unit(1024 * 1024, Unit::Iec(IecSymbol::MiB)),
            (1_f64, Unit::Iec(IecSymbol::MiB))
        );
    }

    #[test]
    fn test_unit_to_bytes() {
        assert_eq!(1024, unit_to_bytes(1, Unit::Iec(IecSymbol::KiB)));
        assert_eq!(1000, unit_to_bytes(1, Unit::Si(SiSymbol::K)));
    }
}
