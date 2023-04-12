//! Gets the status (charging, discharging, unknown, full), percentage, remaining time and
//! power consumption (in Watts) of the given battery and when it's estimated to be empty.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `number` | Battery index as reported in `/sys` | An integer | 0
//! `path` | Set this property if your battery is represented in a non-standard path in `/sys`. The first occurrence of `%d` gets replaced with `number`, but you can also hardcode the battery index in the path | String representing a path | `"/sys/class/power_supply/BAT%d/uevent"`
//! `format` | A string used to customize the output of this module | See available placeholders below | `"$status $percentage $remaining"`
//! `format_down` | A string to customize the output when there are no metrics | A string of UTF-8 characters | `"No battery"`
//! `status_chr` | Custom string to represent `charging` state. | String with any UTF-8 symbols | `"CHR"`
//! `status_bat` | same as above, but for `discharging`. | same as above | `"BAT"`
//! `status_unk` | same as above, but for `unknown` | same as above | `"UNK"`
//! `status_full` | same as above, but for `full` | same as above | `"FULL"`
//! `low_threshold` | Causes the widget state to change to [`WidgetState::Critical`] | Integer | `10`
//! `threshold_type` | An integer representing either time or percentage. If you define `percentage` and your battery percentage goes below `low_threshold`, the widget state will change accordingly. | `time`, `percentage` | `percentage`
//!
//! Placeholder | Value
//! ------------|-------
//! `$status` | Status (charging, discharging, unknown, full) of the battery. Will be replaced by one of the `status_*` properties defined above.
//! `$percentage` | Battery left percentage
//! `$remaining` | Remaining time
//! `$emptytime` | Time when it's estimated to be empty
//! `$consumption` | Power consumption in Watts
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "battery"
//! number = 0
//! format = "$status $percentage $remaining"
//! format_down = "No battery"
//! status_chr = "⚇ CHR"
//! status_bat = "⚡ BAT"
//! status_full = "☻ FULL"
//! low_threshold = 20
//! threshold_type = "percentage"
//! ```
//!
use std::str::FromStr;

use chrono::{prelude::*, Duration};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default = 0]
    pub number: usize,
    #[default("/sys/class/power_supply/BAT%d/uevent")]
    pub path: String,
    #[default(Format::new().with_default("$status $percentage $remaining"))]
    pub format: Format,
    #[default(Format::new().with_default("No battery"))]
    pub format_down: Format,
    #[default("CHR".into())]
    pub status_chr: String,
    #[default("BAT".into())]
    pub status_bat: String,
    #[default("UNK".into())]
    pub status_unk: String,
    #[default("FULL".into())]
    pub status_full: String,
    #[default(10)]
    pub low_threshold: i64,
    pub threshold_type: ThresholdType,
    last_full_capacity: bool,
    hide_seconds: bool,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new();
    let mut timer = bridge.timer().start();
    let path = config.path.replace("%d", &config.number.to_string());

    loop {
        let reader = buffered_reader(&path).await?;
        let psi = PowerSupplyInfo::new(reader, config.last_full_capacity).await?;
        let bi = BatteryInfo::from(&psi);

        if bi.full <= 0 && psi.remaining <= 0 && psi.percentage_remaining <= 0 {
            widget.set_format(config.format_down.clone());
        } else {
            widget.set_format(config.format.clone());
            widget.set_state(if is_battery_low(&config, &bi) {
                WidgetState::Critical
            } else {
                WidgetState::Normal
            });

            widget.set_placeholders(map!(
                "$status" => Value::Text(battery_status(&config, &bi).to_owned()),
                "$percentage" => Value::percentage(bi.percentage_remaining),
                "$remaining" => Value::Text(bi.time_remaining),
                "$emptytime" => Value::Text(bi.empty_time),
                "$consumption" => Value::Text(bi.consumption)
            ));
        }

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

fn is_battery_low(config: &Config, bi: &BatteryInfo) -> bool {
    if config.low_threshold <= 0 || bi.status != ChargingStatus::Discharging {
        return false;
    }

    match config.threshold_type {
        ThresholdType::Percentage if bi.percentage_remaining >= 0_f64 => {
            (bi.percentage_remaining as i64) < config.low_threshold
        }
        ThresholdType::Time if bi.seconds_remaining >= 0_f64 => {
            (bi.seconds_remaining as i64) < 60 * config.low_threshold
        }
        _ => false,
    }
}

fn battery_status<'a>(config: &'a Config, info: &BatteryInfo) -> &'a str {
    match info.status {
        ChargingStatus::Unknown => &config.status_unk,
        ChargingStatus::Charging => &config.status_chr,
        ChargingStatus::Discharging => &config.status_bat,
        ChargingStatus::Full => &config.status_full,
    }
}

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThresholdType {
    Time,
    #[default]
    Percentage,
}

#[derive(Clone, Debug, SmartDefault)]
struct BatteryInfo {
    full: i64,
    status: ChargingStatus,
    percentage_remaining: f64,
    seconds_remaining: f64,
    time_remaining: String,
    empty_time: String,
    consumption: String,
}

impl From<&PowerSupplyInfo> for BatteryInfo {
    fn from(psi: &PowerSupplyInfo) -> Self {
        let mut bi = BatteryInfo::default();
        bi.status = psi.status;

        bi.full = if psi.full_design <= 0 || (psi.last_full_capacity && psi.full_last > 0) {
            psi.full_last
        } else {
            psi.full_design
        };

        bi.percentage_remaining = if psi.percentage_remaining < 0 {
            let p = (psi.remaining as f64 / bi.full as f64) * 100_f64;
            if psi.last_full_capacity && p > 100_f64 {
                100_f64
            } else {
                p
            }
        } else {
            psi.percentage_remaining as f64
        };

        bi.seconds_remaining = if psi.seconds_remaining < 0
            && psi.present_rate > 0
            && bi.status != ChargingStatus::Full
        {
            match bi.status {
                ChargingStatus::Charging => {
                    (3600 * (bi.full - psi.remaining)) as f64 / psi.present_rate as f64
                }
                ChargingStatus::Discharging => {
                    (3600 * psi.remaining) as f64 / psi.present_rate as f64
                }
                _ => 0_f64,
            }
        } else {
            psi.seconds_remaining as f64
        };

        bi.empty_time = {
            let et = Local::now() + Duration::seconds(bi.seconds_remaining as i64);
            format!("{}", et.format("%H:%M:%S"))
        };

        if bi.seconds_remaining >= 0_f64 {
            let hours = (bi.seconds_remaining / 3600_f64) as i64;
            let mut seconds = bi.seconds_remaining as i64 - (hours * 3600);
            let minutes = seconds / 60;
            seconds -= minutes * 60;
            bi.time_remaining = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        }

        bi.consumption = format!("{:1.2}W", psi.present_rate as f64 / 1.0E+6_f64);

        bi
    }
}

#[derive(Debug, Clone, SmartDefault)]
struct PowerSupplyInfo {
    #[default(-1)]
    full_last: i64,
    #[default(-1)]
    full_design: i64,
    #[default(-1)]
    remaining: i64,
    #[default(-1)]
    present_rate: i64,
    #[default(-1)]
    seconds_remaining: i32,
    #[default(-1)]
    voltage: i64,
    #[default(-1)]
    percentage_remaining: i32,
    status: ChargingStatus,
    watt_as_unit: bool,
    last_full_capacity: bool,
}

impl PowerSupplyInfo {
    /// Reference: https://www.kernel.org/doc/html/latest/power/power_supply_class.html
    #[cfg(target_os = "linux")]
    async fn new<T>(mut reader: BufReader<T>, last_full_capacity: bool) -> Result<Self>
    where
        T: AsyncRead + Unpin,
    {
        let mut info = PowerSupplyInfo {
            last_full_capacity,
            ..Default::default()
        };

        let mut line = String::new();

        // Ok(0) means that EOF was reached
        while reader
            .read_line(&mut line)
            .await
            .error("Error reading line.")?
            != 0
        {
            if line.is_empty() {
                continue;
            }

            let mut s = line.trim().split('=');
            let field = s.next().error("No field")?;
            let value = s.next().error("No value")?;

            match field {
                // Momentary energy value
                "POWER_SUPPLY_ENERGY_NOW" => {
                    info.watt_as_unit = true;
                    info.remaining = from_str!(i64, value, field);
                    info.percentage_remaining = -1;
                }
                // Momentary charge value
                "POWER_SUPPLY_CHARGE_NOW" => {
                    info.watt_as_unit = false;
                    info.remaining = from_str!(i64, value, field);
                    info.percentage_remaining = -1;
                }
                // Attribute represents capacity in percents (from 0 to 100)
                "POWER_SUPPLY_CAPACITY" => {
                    if info.remaining == -1 {
                        info.percentage_remaining = from_str!(i32, value, field)
                    }
                }
                "POWER_SUPPLY_CURRENT_NOW" => info.present_rate = from_str!(i64, value, field),
                // Momentary power supply voltage value.
                "POWER_SUPPLY_VOLTAGE_NOW" => info.voltage = from_str!(i64, value, field),
                // Seconds left for battery to be considered empty (i.e. while
                // battery powers a load)
                "POWER_SUPPLY_TIME_TO_EMPTY_NOW" => {
                    info.seconds_remaining = from_str!(i32, value, field) * 60
                }
                "POWER_SUPPLY_POWER_NOW" => info.present_rate = from_str!(i64, value, field),
                // See [`ChargingStatus`]
                "POWER_SUPPLY_STATUS" => info.status = ChargingStatus::from(value),
                // Design charge and energy values, respectively, when battery
                // considered full.
                "POWER_SUPPLY_CHARGE_FULL_DESIGN" | "POWER_SUPPLY_ENERGY_FULL_DESIGN" => {
                    info.full_design = from_str!(i64, value, field)
                }
                // Last remembered value of energy and charge, respectively, when battery
                // became full. These attributes represent real thresholds, not design
                // values (i.e. depend on conditions such temperature or age).
                "POWER_SUPPLY_ENERGY_FULL" | "POWER_SUPPLY_CHARGE_FULL" => {
                    info.full_last = from_str!(i64, value, field)
                }
                _ => (),
            }

            line.clear();
        }

        Ok(info)
    }

    #[cfg(target_os = "freebsd")]
    async fn new(_path: &str) -> Result<Self> {
        todo!()
    }

    #[cfg(target_os = "dragonfly")]
    async fn new(_path: &str) -> Result<Self> {
        todo!()
    }

    #[cfg(target_os = "netbsd")]
    async fn new(_path: &str) -> Result<Self> {
        todo!()
    }

    #[cfg(target_os = "openbsd")]
    async fn new(_path: &str) -> Result<Self> {
        todo!()
    }
}

/// Represents the operating status. Corresponds to
/// `POWER_SUPPLY_STATUS_*`, as defined in [`power_supply.h`].
///
/// [`power_supply.h`]: https://github.com/torvalds/linux/blob/master/include/linux/power_supply.h
#[derive(Clone, Copy, Debug, SmartDefault, Eq, PartialEq)]
pub enum ChargingStatus {
    Unknown,
    Charging,
    #[default]
    Discharging,
    Full,
}

impl From<&str> for ChargingStatus {
    fn from(status: &str) -> Self {
        match status {
            "Charging" => Self::Charging,
            "Discharging" | "Not charging" => Self::Discharging,
            "Full" => Self::Full,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_power_supply() {
        let raw = vec![
            "POWER_SUPPLY_NAME=BAT0",
            "POWER_SUPPLY_TYPE=Battery",
            "POWER_SUPPLY_STATUS=Discharging",
            "POWER_SUPPLY_PRESENT=1",
            "POWER_SUPPLY_TECHNOLOGY=Li-poly",
            "POWER_SUPPLY_CYCLE_COUNT=0",
            "POWER_SUPPLY_VOLTAGE_MIN_DESIGN=11550000",
            "POWER_SUPPLY_VOLTAGE_NOW=11358000",
            "POWER_SUPPLY_CURRENT_NOW=278000",
            "POWER_SUPPLY_CHARGE_FULL_DESIGN=4687000",
            "POWER_SUPPLY_CHARGE_FULL=4687000",
            "POWER_SUPPLY_CHARGE_NOW=1762000",
            "POWER_SUPPLY_CAPACITY=37",
            "POWER_SUPPLY_CAPACITY_LEVEL=Normal",
            "POWER_SUPPLY_MODEL_NAME=DELL J7H5M26",
            "POWER_SUPPLY_MANUFACTURER=SMP",
            "POWER_SUPPLY_SERIAL_NUMBER=1390",
        ]
        .join("\n");
        let data = raw.as_bytes();
        let input = Cursor::new(data);
        let reader = BufReader::new(input);
        let psi = PowerSupplyInfo::new(reader, false).await.unwrap();
        assert_eq!(psi.percentage_remaining, 37.59334328995092);
        assert_eq!(psi.status, ChargingStatus::Discharging);
    }
}
