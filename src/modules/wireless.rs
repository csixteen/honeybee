use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    interface: String,
    #[default(Format::new().with_default("W: ($quality at $essid, $bitrate / $frequency) $ip"))]
    format_up: Format,
    #[default("W: down")]
    format_down: String,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new();
    let mut timer = bridge.timer().start();

    loop {
        widget.set_placeholders(map!(
            "$quality" => Value::Text("CHANGE_ME"),
            "$signal" => Value::Text("CHANGE_ME"),
            "$noise" => Value::Text("CHANGE_ME"),
            "$essid" => Value::Text("CHANGE_ME"),
            "$frequency" => Value::Text("CHANGE_ME"),
            "$ip" => Value::Text("CHANGE_ME"),
            "$bitrate" => Value::Text("CHANGE_ME")
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

const IW_ESSID_MAX_SIZE: usize = 32;

#[derive(Clone, Debug, SmartDefault)]
struct WirelessInfo {
    flags: u32,
    essid: String,
    bssid: [u8; libc::ETH_ALEN as usize],
    quality: i32,
    quality_max: i32,
    quality_avg: i32,
    signal_level: i32,
    signal_level_max: i32,
    noise_level: i32,
    noise_level_max: i32,
    bitrate: u64,
    frequency: f64,
}

impl WirelessInfo {
    fn new() -> Self {
        Self {
            essid: String::with_capacity(IW_ESSID_MAX_SIZE),
            ..Default::default()
        }
    }
}
