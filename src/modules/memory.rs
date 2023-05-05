//! Memory usage on a Linux system from /proc/meminfo.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `format` | A string used to customize the output of this module | See available placeholders below | `"$percentage_used"`
//! `format_degraded` | A string to customize the output when state is set to warning | See available placeholders below | `"$percentage_available"`
//! `threshold_degraded` | Value used to set the state to warning, if available memory falls below given value | Possible values are percentages or exact values followed by unit | `"10%"`
//! `threshold_critical` | Value used to set the state to critical, if available memory falls below given value | Possible values are percentages or exact values followed by unit | `"5%"`
//! `memory_used_method` | Method used to distinguish the actually used memory | See values below | `"classical"`
//! `unit` | IEC unit to be used | See possible values below | `"GiB"`
//! `decimals` | Number of decimals in the format placeholder | An integer number | `1`
//!
//! Memory used method | Value
//! -------------------|-------
//! `classical` | Total memory - free - buffers - cache (matches Gnome system monitor)
//! `memavailable` | Total memory - MemAvailable (matches the `free` command)
//!
//! Placeholder | Value
//! ------------|-------
//! `$total` | Total physical RAM available
//! `$used` | Memory used, based on the chosen method
//! `$percentage_used` | As above, but percentage
//! `$free` | The sum of free low memory (Kernel space) and high memory (User space)
//! `$percentage_free` | As above, but percentage
//! `$available` | An estimate of how much memory is available for starting new applications, without swapping.
//! `$percentage_available` | As above, but percentage
//! `$shared` | Amount of memory consumed in `tmpfs` filesystems.
//! `$percentage_shared` | As above, but percentage
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "memory"
//! format = "$used"
//! threshold_degraded = "85%"
//! threshold_critical = "30 GiB"
//! unit = "GiB"
//! ```

use crate::units::unit_to_bytes;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default(Format::new().with_default("$percentage_used"))]
    format: Format,
    #[default(Format::new().with_default("$percentage_available"))]
    format_degraded: Format,
    #[default(Some(String::from("10%")))]
    threshold_degraded: Option<String>,
    #[default(Some(String::from("5%")))]
    threshold_critical: Option<String>,
    #[default(Some(Default::default()))]
    memory_used_method: Option<MemoryUsedMethod>,
    #[default(Some(IecSymbol::GiB))]
    unit: Option<IecSymbol>,
    #[default(Some(1))]
    decimals: Option<usize>,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new();
    let mut timer = bridge.timer().start();
    let unit = config.unit.expect("unit");
    let decimals = config.decimals.expect("decimals");

    loop {
        widget.set_state(WidgetState::Normal);
        widget.set_format(config.format.clone());

        let meminfo = MemInfo::new().await?;

        if let Some(degraded) = &config.threshold_degraded {
            let threshold = memory_absolute(degraded, meminfo.ram_total)?;
            if meminfo.ram_available < threshold {
                widget.set_format(config.format_degraded.clone());
                widget.set_state(WidgetState::Warning);
            }
        }

        if let Some(critical) = &config.threshold_critical {
            let threshold = memory_absolute(critical, meminfo.ram_total)?;
            if meminfo.ram_available < threshold {
                widget.set_format(config.format_degraded.clone());
                widget.set_state(WidgetState::Critical);
            }
        }

        let ram_used = meminfo.ram_used(
            config
                .memory_used_method
                .as_ref()
                .expect("memory_used_method"),
        );

        widget.set_placeholders(map!(
            "$total" => Value::byte(meminfo.ram_total, Unit::Iec(unit), decimals),
            "$used" => Value::byte(ram_used, Unit::Iec(unit), decimals),
            "$free" => Value::byte(meminfo.ram_free, Unit::Iec(unit), decimals),
            "$available" => Value::byte(meminfo.ram_available, Unit::Iec(unit), decimals),
            "$shared" => Value::byte(meminfo.ram_shared, Unit::Iec(unit), decimals),
            "$percentage_free" => Value::percentage(100_f64 * (meminfo.ram_free as f64 / meminfo.ram_total as f64)),
            "$percentage_available" => Value::percentage(100_f64 * (meminfo.ram_available as f64 / meminfo.ram_total as f64)),
            "$percentage_used" => Value::percentage(100_f64 * (ram_used as f64 / meminfo.ram_total as f64)),
            "$percentage_shared" => Value::percentage(100_f64 * (meminfo.ram_shared as f64 / meminfo.ram_total as f64))
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

// Convert a string to its absolute representation based on the total
// memory of `mem_total`.
//
// The string can contain any percentage values, which then return the
// value of `mem_amount` in relation to `mem_total`. Alternatively, an
// absolute value can be given, suffixed with an IEC symbol.
fn memory_absolute(mem_amount: &str, mem_total: u64) -> Result<u64> {
    let (digits, unit): (String, String) = mem_amount.chars().partition(|c| c.is_ascii_digit());
    let amount = from_str!(u64, &digits, "Bad threshold string");
    let unit = unit
        .trim_start()
        .chars()
        .next()
        .error("Bad threshold string")?;
    if unit == '%' {
        Ok(amount * mem_total / 100)
    } else {
        Ok(unit_to_bytes(amount, Unit::Iec(IecSymbol::try_from(unit)?)))
    }
}

#[derive(Debug, Clone, Copy, SmartDefault, Eq, PartialEq)]
struct MemInfo {
    ram_total: u64,
    ram_free: u64,
    ram_available: u64,
    ram_buffers: u64,
    ram_cached: u64,
    ram_shared: u64,
}

impl MemInfo {
    pub async fn new() -> Result<Self> {
        let f = File::open("/proc/meminfo")
            .await
            .error("Couldn't open /proc/meminfo")?;
        let mut file_reader = BufReader::new(f);
        let mut m = MemInfo::default();
        let mut line = String::new();

        // Ok(0) means that EOF was reached
        while file_reader
            .read_line(&mut line)
            .await
            .error("Couldn't read /proc/meminfo")?
            != 0
        {
            if line.is_empty() {
                continue;
            }

            let mut s = line.split_whitespace();
            let field = s.next().error("No field")?;
            let value = s
                .next()
                .and_then(|v| u64::from_str(v).ok())
                .error("No value")?;

            // All the values are in KB, so we convert them to B
            match field {
                "MemTotal:" => m.ram_total = value * 1024,
                "MemFree:" => m.ram_free = value * 1024,
                "MemAvailable:" => m.ram_available = value * 1024,
                "Buffers:" => m.ram_buffers = value * 1024,
                "Cached:" => m.ram_cached = value * 1024,
                "Shmem:" => m.ram_shared = value * 1024,
                _ => (),
            }

            line.clear();
        }

        Ok(m)
    }

    fn ram_used(&self, used_method: &MemoryUsedMethod) -> u64 {
        match used_method {
            MemoryUsedMethod::Classical => {
                self.ram_total - self.ram_free - self.ram_buffers - self.ram_cached
            }
            MemoryUsedMethod::MemAvailable => self.ram_total - self.ram_available,
        }
    }
}

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
enum MemoryUsedMethod {
    #[default]
    Classical,
    MemAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memoryu_absolute() {
        assert_eq!(50, memory_absolute("50 %".into(), 100).unwrap());
        assert_eq!(50, memory_absolute("50%".into(), 100).unwrap());
        assert_eq!(1024, memory_absolute("1K".into(), 100).unwrap());
        assert_eq!(1024, memory_absolute("1k".into(), 100).unwrap());
        assert_eq!(1024, memory_absolute("1 K".into(), 100).unwrap());
    }
}
