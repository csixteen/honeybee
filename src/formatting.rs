//! Everything related to format strings, placeholders and value types.
use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::str::FromStr;

use serde::de::{MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use smart_default::SmartDefault;

use crate::errors::*;
use crate::pango::*;
use crate::units::{bitrate, bytes_to_unit, Hertz, Unit};

/// Format strings for `full_text` and `short_text`.
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq)]
pub struct Format {
    pub full_text: Option<String>,
    pub short_text: Option<String>,
}

impl Format {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_default(self, full: &str) -> Self {
        self.with_defaults(full, "")
    }

    pub fn with_defaults(self, full: &str, short: &str) -> Self {
        let full_text = self.full_text.or_else(|| Some(full.into()));
        let short_text = self.short_text.or_else(|| Some(short.into()));

        Self {
            full_text,
            short_text,
        }
    }

    fn render_string(format: Option<String>, placeholders: &Option<Placeholders>) -> String {
        match (format, placeholders) {
            (Some(f), Some(p)) => p.iter().fold(f, |haystack, (needle, v)| {
                haystack.replace(needle, &v.to_string())
            }),
            (Some(f), None) => f,
            _ => String::new(),
        }
    }

    /// Replaces the placeholders with the corresponding values in the format strings. Returns
    /// a tuple with the rendered (`full_text`, `short_text`).
    pub fn render(&self, placeholders: &Option<Placeholders>) -> (String, String) {
        let full_text: String = Format::render_string(self.full_text.clone(), placeholders)
            .chars()
            .escape_pango_format();
        let short_text: String = Format::render_string(self.short_text.clone(), placeholders)
            .chars()
            .escape_pango_format();
        (full_text, short_text)
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(full: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Format {
            full_text: Some(full.to_owned()),
            short_text: None,
        })
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Full,
            Short,
        }

        struct FormatVisitor;

        impl<'de> Visitor<'de> for FormatVisitor {
            type Value = Format;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("A format string.")
            }

            fn visit_str<E>(self, full: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                full.parse().serde_error()
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut full_text: Option<String> = None;
                let mut short_text: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Full => {
                            if full_text.is_some() {
                                return Err(de::Error::duplicate_field("full"));
                            }
                            full_text = Some(map.next_value().serde_error()?);
                        }
                        Field::Short => {
                            if short_text.is_some() {
                                return Err(de::Error::duplicate_field("short"));
                            }
                            short_text = Some(map.next_value().serde_error()?);
                        }
                    }
                }

                Ok(Format {
                    full_text,
                    short_text,
                })
            }
        }

        deserializer.deserialize_any(FormatVisitor)
    }
}

/// Maps specific placeholders to the corresponding values. E.g.
/// $total => "32 GiB"
pub type Placeholders = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Icon(String),
    Text(String),
    Number {
        value: f64,
        decimals: usize,
    },
    Percentage(f64),
    Byte {
        value: u64,
        unit: Unit,
        decimals: usize,
    },
    Bits(f64),
    Hertz {
        value: f64,
        unit: Hertz,
        decimals: usize,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Value::Icon(_) => todo!(),
            Value::Text(t) => write!(f, "{}", t),
            Value::Number { value, decimals } => write!(f, "{0:.1$}", value, decimals),
            Value::Percentage(p) => write!(f, "{:.2}%", p),
            Value::Byte {
                value,
                unit,
                decimals,
            } => {
                let (v, u) = bytes_to_unit(*value, *unit);
                write!(f, "{0:.2$} {1}", v, u, decimals)
            }
            Value::Bits(b) => write!(f, "{}", bitrate(*b)),
            Value::Hertz {
                value,
                unit,
                decimals,
            } => write!(f, "{0:.1$} {2}", value, decimals, unit),
        }
    }
}

impl Value {
    pub fn byte(value: u64, unit: Unit, decimals: usize) -> Value {
        Self::Byte {
            value,
            unit,
            decimals,
        }
    }

    pub fn hertz(value: f64, unit: Hertz, decimals: usize) -> Value {
        Self::Hertz {
            value,
            unit,
            decimals,
        }
    }

    pub fn bits(value: f64) -> Value {
        Self::Bits(value)
    }

    pub fn percentage(value: f64) -> Value {
        Self::Percentage(value)
    }

    pub fn number(value: f64, decimals: usize) -> Value {
        Self::Number { value, decimals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_defaults() {
        let f = Format {
            full_text: Some("foo".into()),
            short_text: Some("bar".into()),
        }
        .with_defaults("foo2", "bar2");

        assert_eq!(Some("foo".into()), f.full_text);
        assert_eq!(Some("bar".into()), f.short_text);
    }
}
