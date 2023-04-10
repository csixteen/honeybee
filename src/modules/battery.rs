use std::str::FromStr;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    number: usize,
    path: String,
    #[default(Format::new().with_default("$status $percentage $remaining"))]
    format: Format,
    #[default(Format::new().with_default("No battery"))]
    format_down: Format,
    #[default("CHR".into())]
    status_chr: String,
    #[default("BAT".into())]
    status_bat: String,
    #[default("UNK".into())]
    status_unk: String,
    #[default("FULL".into())]
    status_full: String,
    low_threshold: usize,
    threshold_type: ThresholdType,
    last_full_capacity: bool,
    format_percentage: Format,
    hide_seconds: bool,
}

pub(crate) async fn run(mut config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new();
    let format = std::mem::take(&mut config.format);
    let mut timer = bridge.timer().start();
    let path = config.path.replace("%d", &config.number.to_string());

    loop {
        widget.set_state(WidgetState::Normal);
        widget.set_format(format.clone());

        let psi = PowerSupplyInfo::new(&path).await?;

        widget.set_placeholders(map!(
            "$status" => Value::Text(battery_status(&config, &psi).to_owned())
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

fn battery_status<'a>(config: &'a Config, info: &PowerSupplyInfo) -> &'a str {
    match info.status {
        ChargingStatus::Unknown => &config.status_unk,
        ChargingStatus::Charging => &config.status_chr,
        ChargingStatus::Discharging | ChargingStatus::NotCharging => &config.status_bat,
        ChargingStatus::Full => &config.status_full,
    }
}

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
pub enum ThresholdType {
    Time,
    #[default]
    Percentage,
}

#[derive(Debug, Clone, SmartDefault)]
struct PowerSupplyInfo {
    #[default(-1_f64)]
    full_design: f64,
    #[default(-1_f64)]
    full_last: f64,
    #[default(-1_f64)]
    remaining: f64,
    #[default(-1_f64)]
    present_rate: f64,
    #[default(-1)]
    seconds_remaining: i32,
    #[default(-1)]
    percentage_remaining: i32,
    status: ChargingStatus,
}

impl PowerSupplyInfo {
    /// Reference: https://www.kernel.org/doc/html/latest/power/power_supply_class.html
    #[cfg(target_os = "linux")]
    async fn new(path: &str) -> Result<Self> {
        let f = File::open(path)
            .await
            .or_error(|| format!("Couldn't open {path}"))?;
        let mut file_reader = BufReader::new(f);
        let mut info = PowerSupplyInfo::default();
        let mut line = String::new();
        let mut watt_as_unit = false;
        let mut voltage = -1;

        // Ok(0) means that EOF was reached
        while file_reader
            .read_line(&mut line)
            .await
            .or_error(|| format!("Couldn't read {path}"))?
            != 0
        {
            if line.is_empty() {
                continue;
            }

            let mut s = line.split("=");
            let field = s.next().error("No field")?;
            let value = s.next().error("No value")?;

            match field {
                // Momentary energy value
                "POWER_SUPPLY_ENERGY_NOW" => {
                    watt_as_unit = true;
                    info.remaining = from_str!(f64, value, field)
                }
                // Momentary charge value
                "POWER_SUPPLY_CHARGE_NOW" => {
                    watt_as_unit = false;
                    info.remaining = from_str!(f64, value, field)
                }
                // Attribute represents capacity in percents (from 0 to 100)
                "POWER_SUPPLY_CAPACITY" => info.percentage_remaining = from_str!(i32, value, field),
                "POWER_SUPPLY_CURRENT_NOW" => info.present_rate = from_str!(f64, value, field),
                // Momentary power supply voltage value.
                "POWER_SUPPLY_VOLTAGE_NOW" => voltage = from_str!(i32, value, field),
                // Seconds left for battery to be considered empty (i.e. while
                // battery powers a load)
                "POWER_SUPPLY_TIME_TO_EMPTY_NOW" => {
                    info.seconds_remaining = from_str!(i32, value, field)
                }
                "POWER_SUPPLY_POWER_NOW" => info.present_rate = from_str!(f64, value, field),
                // See [`ChargingStatus`]
                "POWER_SUPPLY_STATUS" => info.status = ChargingStatus::from(value),
                // Design charge and energy values, respectively, when battery
                // considered full.
                "POWER_SUPPLY_CHARGE_FULL_DESIGN" | "POWER_SUPPLY_ENERGY_FULL_DESIGN" => {
                    info.full_design = from_str!(f64, value, field)
                }
                // Last remembered value of energy and charge, respectively, when battery
                // became full. These attributes represent real thresholds, not design
                // values (i.e. depend on conditions such temperature or age).
                "POWER_SUPPLY_ENERGY_FULL" | "POWER_SUPPLY_CHARGE_FULL" => {
                    info.full_last = from_str!(f64, value, field)
                }
                _ => (),
            }
        }

        if !watt_as_unit && voltage >= 0 {
            let voltage = voltage as f64;

            if info.present_rate > 0_f64 {
                info.present_rate = (voltage / 1000_f64) * (info.present_rate / 1000_f64);
            }
            if info.remaining > 0_f64 {
                info.remaining = (voltage / 1000_f64) * (info.remaining / 1000_f64);
            }
            if info.full_design > 0_f64 {
                info.full_design = (voltage / 1000_f64) * (info.full_design / 1000_f64);
            }
            if info.full_last > 0_f64 {
                info.full_last = (voltage / 1000_f64) * (info.full_last / 1000_f64);
            }
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
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq)]
pub enum ChargingStatus {
    Unknown,
    Charging,
    #[default]
    Discharging,
    NotCharging,
    Full,
}

impl From<&str> for ChargingStatus {
    fn from(status: &str) -> Self {
        match status {
            "Charging" => Self::Charging,
            "Discharging" => Self::Discharging,
            "Not charging" => Self::NotCharging,
            "Full" => Self::Full,
            _ => Self::Unknown,
        }
    }
}
