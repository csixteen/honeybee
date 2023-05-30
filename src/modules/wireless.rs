//! Gets the link signal, frequency and SSID of the given wireless network interface.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|---------
//! `interface` | Wireless network interface to monitor (as specified in `/sys/class/net/`) | A string | `"wlan0"`
//! `format_up` | String to customize the output of the module when the interface is up. See placeholders below | `"$name: [$signal at $ssid, $bitrate [D: $speed_down | U: $speed_up]] $ipv4"`
//! `format_down` | String to customize the output of the module when the interface is down. | `"W: down"`
//!
//! Placeholder | Value
//! ------------|-------
//! `$name` | Name of the Wireless network interface.
//! `$ipv4` | IPv4 address of the interface
//! `$ipv6` | IPv6 address of the interface
//! `$ssid` | Network SSID
//! `$signal` | WiFi signal strength
//! `$bitrate` | WiFi connection bitrate
//! `$frequency` | Interface frequency of the selected channel (Ghz)
//! `$speed_up` | Upload speed
//! `$speed_down` | Download speed
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "wireless"
//! interface = "wlan0"
//! ```
//!

use std::collections::HashMap;
use std::time::Instant;

use super::prelude::*;
use crate::formatting::Placeholders;
use crate::net_iface::{NetworkInterface, RtnlLinkStats};

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default("wlan0")]
    interface: String,
    #[default(Format::new().with_default("$name: [$signal at $ssid, $bitrate [D: $speed_down | U: $speed_up]] $ipv4"))]
    format_up: Format,
    #[default(Format::new().with_default("W: down"))]
    format_down: Format,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut timer = bridge.timer().start();

    let mut rx_speed = 0f64;
    let mut tx_speed = 0f64;
    let mut stats: Option<RtnlLinkStats> = None;
    let mut stats_timer = Instant::now();

    loop {
        let mut widget = Widget::new();

        match NetworkInterface::new(&config.interface).await? {
            None => {
                widget.set_state(WidgetState::Critical);
                widget.set_format(config.format_down.clone());
            }
            Some(iface) => {
                widget.set_state(WidgetState::Normal);
                widget.set_format(config.format_up.clone());

                let mut ph: Placeholders = HashMap::new();
                ph.insert("$name".to_string(), Value::Text(config.interface.clone()));
                ph.insert(
                    "$ipv4".to_string(),
                    Value::Text(
                        iface
                            .ipv4
                            .map(|ip| ip.to_string())
                            .unwrap_or("No IPv4".to_string()),
                    ),
                );

                ph.insert(
                    "$ipv6".to_string(),
                    Value::Text(
                        iface
                            .ipv6
                            .map(|ip| ip.to_string())
                            .unwrap_or("No IPv6".to_string()),
                    ),
                );

                ph.insert(
                    "$ssid".to_string(),
                    Value::Text(iface.ssid().unwrap_or("No SSID".to_string())),
                );

                ph.insert(
                    "$signal".to_string(),
                    iface
                        .signal()
                        .map(Value::percentage)
                        .unwrap_or(Value::Text("No signal".to_string())),
                );

                ph.insert(
                    "$bitrate".to_string(),
                    iface
                        .bitrate()
                        .map(Value::bits)
                        .unwrap_or(Value::Text("N/A".to_string())),
                );

                ph.insert(
                    "$frequency".to_string(),
                    iface
                        .frequency()
                        .map(|f| Value::hertz(f / 1e9, Hertz::GHz, 2))
                        .unwrap_or(Value::Text("N/A".to_string())),
                );

                match (stats, iface.stats) {
                    (Some(_), None) => stats = None,
                    (None, new_stats) => stats = new_stats,
                    (Some(old_stats), Some(new_stats)) => {
                        let elapsed = stats_timer.elapsed().as_secs_f64();
                        stats_timer = Instant::now();
                        rx_speed = (new_stats.rx_bytes - old_stats.rx_bytes) as f64 / elapsed;
                        tx_speed = (new_stats.tx_bytes - old_stats.tx_bytes) as f64 / elapsed;
                    }
                }

                ph.insert(
                    "$speed_up".to_string(),
                    Value::byte(tx_speed as u64, Unit::iec_from_char('m'), 2),
                );
                ph.insert(
                    "$speed_down".to_string(),
                    Value::byte(rx_speed as u64, Unit::iec_from_char('m'), 2),
                );

                widget.set_placeholders(ph);
            }
        }

        bridge.set_widget(widget).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}
