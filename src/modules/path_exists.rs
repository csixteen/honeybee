//! Check if the given `path` exists in the filesystem. You can use this to check if
//! something is active, like for example a VPN connection managed by NetworkManager.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `title` | Will replace the placeholder `$title` | A string | N/A
//! `path` | The actual path you want to check if exists. | String representing an absolute path | N/A
//! `format` | A custom string to format the status of the path in the bar | A string with placeholders | `"$title $status"`
//!
//! Placeholder | Value
//! ------------|-------
//! `$title` | A descriptive title for your path (e.g. "VPN")
//! `$status` | A string with the values `yes` and `no`, indicating whether the path exists or not
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "path_exists"
//! title = "VPN"
//! path = "/non/existing/path"
//! ```
use tokio::fs;

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
pub struct Config {
    title: String,
    format: Option<String>,
    path: String,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(config.title.clone());
    let format: Format = match config.format {
        Some(f) => f.parse()?,
        None => "$title $status".parse()?,
    };
    widget.set_format(format);
    let mut timer = bridge.timer().start();

    loop {
        let status = match fs::try_exists(&config.path).await {
            Ok(true) => {
                widget.set_state(WidgetState::Normal);
                "yes"
            }
            _ => {
                widget.set_state(WidgetState::Critical);
                "no"
            }
        };

        widget.set_placeholders(map!(
            "$title" => Value::Text(config.title.clone()),
            "$status" => Value::Text(status.into())
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}
