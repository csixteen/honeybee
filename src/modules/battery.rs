use std::str::FromStr;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

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

        let reader = buffered_reader(&path).await?;
        let psi = PowerSupplyInfo::new(reader, config.last_full_capacity).await?;

        widget.set_placeholders(map!(
            "$status" => Value::Text(battery_status(&config, &psi).to_owned()),
            "$percentage" => Value::percentage(psi.percentage_remaining),
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
    full: f64,
    #[default(-1_f64)]
    remaining: f64,
    #[default(-1_f64)]
    present_rate: f64,
    #[default(-1)]
    seconds_remaining: i32,
    #[default(-1_f64)]
    percentage_remaining: f64,
    status: ChargingStatus,
}

impl PowerSupplyInfo {
    /// Reference: https://www.kernel.org/doc/html/latest/power/power_supply_class.html
    #[cfg(target_os = "linux")]
    async fn new<T>(mut reader: BufReader<T>, last_full_capacity: bool) -> Result<Self>
    where
        T: AsyncRead + Unpin,
    {
        let mut info = PowerSupplyInfo::default();
        let mut line = String::new();
        let mut watt_as_unit = false;
        let mut voltage = -1;
        let mut pct_remaining: i32 = -1;
        let mut full_design = -1_f64;
        let mut full_last = -1_f64;

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
                    watt_as_unit = true;
                    info.remaining = from_str!(f64, value, field);
                    pct_remaining = -1;
                }
                // Momentary charge value
                "POWER_SUPPLY_CHARGE_NOW" => {
                    watt_as_unit = false;
                    info.remaining = from_str!(f64, value, field);
                    pct_remaining = -1;
                }
                // Attribute represents capacity in percents (from 0 to 100)
                "POWER_SUPPLY_CAPACITY" => {
                    if info.remaining == -1_f64 {
                        pct_remaining = from_str!(i32, value, field)
                    }
                }
                "POWER_SUPPLY_CURRENT_NOW" => info.present_rate = from_str!(f64, value, field),
                // Momentary power supply voltage value.
                "POWER_SUPPLY_VOLTAGE_NOW" => {
                    voltage = from_str!(i64, value, format!("{field} / {value}"))
                }
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
                    full_design = from_str!(f64, value, field)
                }
                // Last remembered value of energy and charge, respectively, when battery
                // became full. These attributes represent real thresholds, not design
                // values (i.e. depend on conditions such temperature or age).
                "POWER_SUPPLY_ENERGY_FULL" | "POWER_SUPPLY_CHARGE_FULL" => {
                    full_last = from_str!(f64, value, field)
                }
                _ => (),
            }

            line.clear();
        }

        if !watt_as_unit && voltage >= 0 {
            let voltage = voltage as f64;

            if info.present_rate > 0_f64 {
                info.present_rate = (voltage / 1000_f64) * (info.present_rate / 1000_f64);
            }
            if info.remaining > 0_f64 {
                info.remaining = (voltage / 1000_f64) * (info.remaining / 1000_f64);
            }
            if full_design > 0_f64 {
                full_design = (voltage / 1000_f64) * (full_design / 1000_f64);
            }
            if full_last > 0_f64 {
                full_last = (voltage / 1000_f64) * (full_last / 1000_f64);
            }
        }

        info.full = if full_design <= 0_f64 || (last_full_capacity && full_last > 0_f64) {
            full_last
        } else {
            full_design
        };

        info.percentage_remaining = if pct_remaining < 0 {
            let p = (info.remaining / info.full) * 100_f64;
            if last_full_capacity && p > 100_f64 {
                100_f64
            } else {
                p
            }
        } else {
            pct_remaining as f64
        };

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
        assert_eq!(psi.percentage_remaining, 37);
        assert_eq!(psi.status, ChargingStatus::Discharging);
    }
}
