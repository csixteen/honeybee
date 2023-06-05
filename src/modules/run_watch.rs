//! Expands the given path to a pidfile and checks if the process ID found inside is of
//! a running process. You can use this to check if a specific application, such as a VPN
//! client, is running.
//!
//! # Configuration
//!
//! Key | Description | Values | Default
//! ----|-------------|--------|--------
//! `title` | Will replace the placeholder `$title` | A string | N/A
//! `pidfile` | A path to a pidfile or a pattern that matches to pathnames (see `glob(3)`) | A string | N/A
//! `format` | A custom string to format the status of the process in the bar | A string with placeholders | `"$title: $status"`
//! `format_down` | The same as above for when the process is not running | A string with placeholders | `"$title: $status"`
//!
//! Placeholder | Value
//! ------------|-------
//! `$title` | A descriptive title for your path (e.g. "VPN")
//! `$status` | A string with the values `yes` and `no`, indicating whether the path exists or not
//!
//! # Example
//! ```toml
//! [[module]]
//! module = "run_watch"
//! title = "Spotify"
//! pidfile = "/path/to/some/pidfile"
//!
//! [[module]]
//! module = "run_watch"
//! title = "VPN"
//! pidfile = "/path/to/vpn*.pid"
//! format_down = "$title is down!!!"
//! ```

use glob::glob;
use nix::sys::signal;
use nix::unistd::Pid;
use tokio::fs;

use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
pub struct Config {
    title: String,
    format: Option<String>,
    format_down: Option<String>,
    pidfile: String,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(config.pidfile.clone());
    let format: Format = match config.format {
        Some(f) => f.parse()?,
        None => "$title: $status".parse()?,
    };
    let format_down = match config.format_down {
        Some(f) => f.parse()?,
        None => format.clone(),
    };
    let mut timer = bridge.timer().start();

    loop {
        let status = process_runs(&config.pidfile).await;

        match status {
            Ok(true) => {
                widget.set_state(WidgetState::Normal);
                widget.set_format(format.clone());
            }
            _ => {
                widget.set_state(WidgetState::Critical);
                widget.set_format(format_down.clone());
            }
        }

        widget.set_placeholders(map!(
            "$title" => Value::Text(config.title.clone()),
            "$status" => Value::Text(match status {
                Ok(true) => "yes".into(),
                Ok(false) => "no".into(),
                Err(e) => e.to_string()
            })
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

async fn process_runs(pidfile: &str) -> Result<bool> {
    for path in glob(pidfile)
        .map_err(|e| Error::new("Invalid pidfile").with_source(Arc::new(e)))
        .unwrap()
        .filter_map(Result::ok)
    {
        let p: i32 = fs::read_to_string(path)
            .await
            .error("Couldn't read pid file")?
            .parse::<i32>()
            .error("Invalid pid")?;
        // `None` means signal 0, which means that no signal is actually sent
        // but error checking is performed instead. If the process is running
        // then `res` will be Ok(()).
        let res = signal::kill(Pid::from_raw(p), None);
        if res.is_ok() {
            return Ok(true);
        }
    }

    Ok(false)
}
