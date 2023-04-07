//! Everything needed to manipulate colors.
use std::fmt::{self, Formatter};

use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::errors::*;

/// A color is a string representing a canonical RGB hexadecimal triplet
/// with no separator between colors. E.g. `"#FF0000"`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Color(String);

impl Default for Color {
    fn default() -> Self {
        Color(String::from("#ffffff"))
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Color {
    pub fn new() -> Self {
        Default::default()
    }
}

impl TryFrom<&str> for Color {
    type Error = Error;

    fn try_from(c: &str) -> Result<Self, Self::Error> {
        if c.starts_with('#') && c.len() == 7 {
            u32::from_str_radix(&c[1..], 16).map_err(|_| Error::new("Bad color value"))?;
            Ok(Color(c.to_owned()))
        } else {
            Err(Error::new("Bad color format"))
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("A color in hexadecimal format. E.g. #ff0000")
            }

            fn visit_str<E>(self, color: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Color::try_from(color).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_string_ok() {
        assert!(Color::try_from("#000000").is_ok());
        assert!(Color::try_from("#111111").is_ok());
    }

    #[test]
    fn test_color_from_string_err() {
        assert!(Color::try_from("#lazyfox123").is_err());
        assert!(Color::try_from("#1234567890").is_err());
    }
}
