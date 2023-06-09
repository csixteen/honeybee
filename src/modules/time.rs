//! Outputs the current time in the local timezone, if `timezone` is not set, or in
//! given timezone, if `timezone` is set.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `title` | Identifies this particular time instance (you can have multiple) | A string | N/A
//! `format` | Format string used to define what the output will look like | See below | `"%Y-%m-%d %H:%M %Z"`
//! `timezone` | The current time will be output in the given timezone | See below | `"Local"`
//!
//! # Format
//!
//! The value of `format` should be a string with [`valid identifiers`]. You can also see
//! `strftime(3)` for details.
//!
//! # TimeZone
//!
//! The value of `timezone` can be `Local` (meaning that the local time will be used), `UTC` or
//! any valid [`TZ identifier`].
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "time"
//! title = "Amsterdam"
//! format = "%Y-%m-%d %H:%M %Z"
//! timezone = "Europe/Amsterdam"
//! ```
//!
//! [`valid identifiers`]: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
//! [`TZ identifier`]: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones

use chrono::{Local, Utc};
use chrono_tz::Tz;

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
pub struct Config {
    title: String,
    format: Option<String>,
    #[serde(default)]
    timezone: TimeZone,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(config.title);
    widget.set_format(Format::new().with_default("$time"));
    let mut timer = bridge.timer().start();
    let format = match config.format {
        None => "%Y-%m-%d %H:%M %Z".to_string(),
        Some(f) => f,
    };

    loop {
        let t = match &config.timezone {
            TimeZone::Utc => Utc::now().format(&format),
            TimeZone::Local => Local::now().format(&format),
            TimeZone::TimeZone(tz) => {
                let ttz: Tz = tz.parse().unwrap();
                Utc::now().with_timezone(&ttz).format(&format)
            }
        };

        widget.set_placeholders(map!(
            "$time" => Value::Text(t.to_string()),
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, SmartDefault, Deserialize)]
#[serde(try_from = "String")]
enum TimeZone {
    Utc,
    #[default]
    Local,
    TimeZone(String),
}

impl TryFrom<&str> for TimeZone {
    type Error = Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "utc" | "UTC" | "Utc" => Ok(Self::Utc),
            "local" | "Local" => Ok(Self::Local),
            _ => {
                let _ = Tz::from_str(s).map_err(|_| Error::new("Bad timezone format"))?;
                Ok(Self::TimeZone(s.into()))
            }
        }
    }
}

impl TryFrom<String> for TimeZone {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        TimeZone::try_from(value.as_str())
    }
}
