//! Gets the system load (number of processes waiting for CPU time in the last 1, 5 and
//! 15 minutes).
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `format` | Format string used to define what the output will look like | See placeholders below | `"$1min $5min $15min"
//! `format_above_threshold` | Format used when load average of the last minute is above `max_threshold` | Same as above | `"Warning: $1min"`
//! `max_threshold` | The current time will be output in the given timezone | A double | `5`
//!
//! Placeholder | Value
//! ------------|-------
//! `$1min` | Load average in the last minute.
//! `$5min` | Load average in the last 5 minutes.
//! `$15min` | Load average in the last 15 minutes.
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "load_avg"
//! format = "Avg: $1min"
//! max_threshold = 3
//! ```

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default(Format::new().with_default("$1min $5min $15min"))]
    format: Format,
    format_above_threshold: Option<Format>,
    #[default(5_f64)]
    max_threshold: f64,
}

pub(crate) async fn run(mut config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new();
    let format = std::mem::take(&mut config.format);
    let format_above_threshold = match config.format_above_threshold {
        Some(f) => f,
        None => Format::new().with_default("Warning: $1min"),
    };
    let mut timer = bridge.timer().start();

    loop {
        let mut avg: [libc::c_double; 3] = [-1_f64, -1_f64, -1_f64];
        let ptr = &mut avg as *mut libc::c_double;

        unsafe {
            libc::getloadavg(ptr, 3);
        }

        if avg[0] >= config.max_threshold {
            widget.set_format(format_above_threshold.clone());
            widget.set_state(WidgetState::Critical);
        } else {
            widget.set_format(format.clone());
            widget.set_state(WidgetState::Normal);
        }

        widget.set_placeholders(map!(
            "$1min" => Value::number(avg[0], 1),
            "$5min" => Value::number(avg[1], 1),
            "$15min" => Value::number(avg[2], 1),
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}
