use std::collections::HashMap;

use super::prelude::*;
use crate::formatting::Placeholders;
use crate::net_iface::NetworkInterface;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    interface: String,
    #[default(Format::new().with_default("W: ($signal at $ssid, $bitrate / $frequency) $ipv4"))]
    format_up: Format,
    #[default(Format::new().with_default("W: down"))]
    format_down: Format,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut timer = bridge.timer().start();

    loop {
        let mut widget = Widget::new();

        match NetworkInterface::new(&config.interface).await? {
            None => {
                widget.set_format(config.format_down.clone());
            }
            Some(iface) => {
                widget.set_format(config.format_up.clone());

                let mut ph: Placeholders = HashMap::new();
                if let Some(ipv4) = iface.ipv4 {
                    ph.insert("$ipv4".to_string(), Value::Text(ipv4.to_string()));
                }

                if let Some(ipv6) = iface.ipv4 {
                    ph.insert("$ipv6".to_string(), Value::Text(ipv6.to_string()));
                }

                if let Some(ssid) = iface.ssid() {
                    ph.insert("$ssid".to_string(), Value::Text(ssid));
                }

                if let Some(signal) = iface.signal() {
                    ph.insert("$signal".to_string(), Value::percentage(signal));
                }

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
