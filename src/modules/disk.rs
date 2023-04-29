//! Gets used, free, available and total amount of bytes on the given mounted filesystem.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `path` | Path to the mounted filesystem | A string representing a path | `"/"`
//! `format` | A string used to customize the output of this module | See available placeholders below | `"$free"`
//! `format_below_threshold` | A string to customize the output when state is set to criticalo | See available placeholders below | `"Warning: $percentage_avail"`
//! `format_not_mounted` | A string to customize the output when the path doesn't exist or is not a mount point. | A string | `""`
//! `prefix_type` | Prefix used to present byte sizes in human readable format | See options below | `"binary"`
//! `threshold_type` | Type of `low_threshold` that sets the widget state to critical | See options below | `"percentage_avail"`
//! `low_threshold` | Value that causes the disk text to be displayed using `color_bad` (widget state critical) | Number | `0`
//!
//! Prefix Type | Value
//! ------------|-------
//! `binary` | IEC prefixes (KiB, MiB, GiB, TiB) represent multiples of powers of 1024
//! `decimal` | SI prefixes (K, M, G, T) represent multiples of powers of 1000
//!
//! Threshold Type | Value
//! ---------------|-------
//! `percentage_free` | Percentage of disk space free
//! `percentage_avail` | Percentage of disk space available
//! `bytes_free` | Number of bytes free
//! `bytes_avail` | Number of bytes available
//!
//!  Note that `bytes_free` and `bytes_avail` can be prefixed with "k", "m", "g" or "t". That means that if
//! you set `prefix_type` to `binary`, `low_threshold` to `2` and `threshold_type` to `gbytes_avail`, then
//! the disk info will be colored bad.
//!
//! If not specified, `threshold_type` is assumed to be `percentage_avail` and `low_threshold` is `0`, which
//! implies no coloring at all.
//!
//! Placeholder | Value
//! ------------|-------
//! `$free` | Free disk space
//! `$percentage_free` | As above, but percentage
//! `$avail` | Available disk space
//! `$percentage_avail` | As above, but percentage
//! `$total` | Total disk space
//! `$used` | Used disk space
//! `$percentage_used` | As above, but percentage
//! `$percentage_used_of_avail` | Percentage of available space being used
//!
//! # Example
//!
//! ```toml
//! [[module]]
//! module = "disk"
//! format = "/: $avail"
//! prefix_type = "binary"
//! threshold_type = "gbytes_avail"
//! low_threshold = 10
//! ```
//!

use nix::sys::statvfs;

use super::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Config {
    #[default("/")]
    path: String,
    #[default(Format::new().with_default("$free"))]
    format: Format,
    #[default(Format::new().with_default("Warning: $percentage_avail"))]
    format_below_threshold: Format,
    #[default(Format::new().with_default(""))]
    format_not_mounted: Format,
    prefix_type: PrefixType,
    threshold_type: ThresholdType,
    low_threshold: f64,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(config.path.clone());
    let mut timer = bridge.timer().start();
    let unit = match config.prefix_type {
        PrefixType::Binary => Unit::iec_from_str("ti"),
        _ => Unit::si_from_str("t"),
    };

    loop {
        match DiskInfo::new(&config.path) {
            Ok(disk_info) => {
                if disk_info.below_threshold(
                    config.prefix_type,
                    config.threshold_type,
                    config.low_threshold,
                ) {
                    widget.set_format(config.format_below_threshold.clone());
                    widget.set_state(WidgetState::Critical);
                } else {
                    widget.set_format(config.format.clone());
                    widget.set_state(WidgetState::Normal);
                }

                widget.set_placeholders(map!(
                    "$free" => Value::byte(disk_info.free, unit, 2),
                    "$used" => Value::byte(disk_info.used, unit, 2),
                    "$total" => Value::byte(disk_info.total, unit, 2),
                    "$avail" => Value::byte(disk_info.available, unit, 2),
                    "$percentage_free" => Value::percentage(disk_info.percentage_free),
                    "$percentage_used_of_avail" => Value::percentage(disk_info.percentage_used_of_avail),
                    "$percentage_used" => Value::percentage(disk_info.percentage_used),
                    "$percentage_avail" => Value::percentage(disk_info.percentage_avail),
                ));
            }
            Err(_) => {
                widget.set_format(config.format_not_mounted.clone());
            }
        }

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, SmartDefault)]
#[serde(rename_all = "lowercase")]
enum PrefixType {
    #[default]
    Binary,
    Decimal,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, SmartDefault)]
#[serde(try_from = "String")]
enum ThresholdType {
    BytesFree,
    BytesAvail,
    PercentageFree,
    #[default]
    PercentageAvail,
    PrefixBytesFree(char),
    PrefixBytesAvail(char),
}

impl TryFrom<String> for ThresholdType {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        ThresholdType::try_from(value.as_str())
    }
}

impl TryFrom<&str> for ThresholdType {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "bytes_free" => Ok(Self::BytesFree),
            "bytes_avail" => Ok(Self::BytesAvail),
            "percentage_free" => Ok(Self::PercentageFree),
            "percentage_avail" => Ok(Self::PercentageAvail),
            _ => {
                let unit = value.chars().next().error("Empty string")?;
                valid_prefix(unit)?;
                let ttype = value.chars().skip(1).collect::<String>();
                match ttype.as_str() {
                    "bytes_free" => Ok(Self::PrefixBytesFree(unit)),
                    "bytes_avail" => Ok(Self::PrefixBytesAvail(unit)),
                    _ => Err(Error::new(format!("Invalid threshold type: {value}"))),
                }
            }
        }
    }
}

#[inline]
fn valid_prefix(p: char) -> Result<()> {
    if !(p == 'k' || p == 'm' || p == 'g' || p == 't') {
        Err(Error::new("Invalid prefix"))
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, SmartDefault)]
struct DiskInfo {
    f_bsize: u64,
    f_bfree: u64,
    f_bavail: u64,
    free: u64,
    used: u64,
    total: u64,
    available: u64,
    percentage_free: f64,
    percentage_used_of_avail: f64,
    percentage_used: f64,
    percentage_avail: f64,
}

impl DiskInfo {
    fn new(path: &str) -> Result<Self> {
        let res = statvfs::statvfs(path).or_error(|| format!("{path} is not mounted."))?;
        let f_blocks = res.blocks();
        let f_bfree = res.blocks_free();
        let f_bavail = res.blocks_available();
        let unit_size = if cfg!(any(target_os = "linux", target_os = "netbsd")) {
            res.fragment_size()
        } else {
            res.block_size()
        };

        Ok(Self {
            f_bsize: res.block_size(),
            f_bfree,
            f_bavail,
            free: unit_size * f_bfree,
            used: unit_size * (f_blocks - f_bfree),
            total: unit_size * f_blocks,
            available: unit_size * f_bavail,
            percentage_free: 100_f64 * (f_bfree as f64 / f_blocks as f64),
            percentage_used_of_avail: 100_f64 * ((f_blocks - f_bavail) as f64) / (f_blocks as f64),
            percentage_used: 100_f64 * ((f_blocks - f_bfree) as f64) / (f_blocks as f64),
            percentage_avail: 100_f64 * (f_bavail as f64 / f_blocks as f64),
        })
    }

    fn below_threshold(&self, ptype: PrefixType, ttype: ThresholdType, low_threshold: f64) -> bool {
        match ttype {
            ThresholdType::PercentageFree => self.percentage_free < low_threshold,
            ThresholdType::PercentageAvail => self.percentage_avail < low_threshold,
            ThresholdType::BytesFree => (self.free as f64) < low_threshold,
            ThresholdType::BytesAvail => (self.available as f64) < low_threshold,
            ThresholdType::PrefixBytesFree(c) => {
                prefixed_below_threshold(ptype, self.f_bsize, self.f_bfree, low_threshold, c)
            }
            ThresholdType::PrefixBytesAvail(c) => {
                prefixed_below_threshold(ptype, self.f_bavail, self.f_bfree, low_threshold, c)
            }
        }
    }
}

#[inline]
fn prefixed_below_threshold(
    ptype: PrefixType,
    value: u64,
    free: u64,
    low_threshold: f64,
    c: char,
) -> bool {
    let unit = match ptype {
        PrefixType::Binary => Unit::iec_from_char(c),
        _ => Unit::si_from_char(c),
    };
    let bytes = unit_to_bytes(low_threshold as u64, unit);
    value * free < bytes
}
