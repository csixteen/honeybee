#[allow(clippy::large_enum_variant)]
pub mod bar;
mod bridge;
pub mod config;
pub mod errors;
mod formatting;
mod macros;
pub mod modules;
pub(crate) mod output;
mod protocol;
pub(crate) mod timer;
pub(crate) mod types;
mod units;
pub mod utils;
mod widget;

use clap::Parser;

/// A port of the original i3status(1) written in Rust. This is mostly a learning
/// exercise, although my aim is to port it completely and, eventually, enhance with
/// other features.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct CliArgs {
    /// If an absolute path is provided, then it shall be used (this also performs
    /// tilde expansion). If it's not an absolute path, or if the file doesn't exist,
    /// then a valid confdiguration will be searched using the following order:
    ///
    /// 1. $XDG_CONFIG_HOME/honeybee/config.toml
    ///
    /// 2. $XDG_DATA_HOME/honeybee/config.toml
    ///
    /// 3. $HOME/.honeybee.toml
    ///
    /// 4. $XDG_DATA_DIRS/honeybee/config.toml
    ///
    /// 5. $XDG_CONFIG_DIRS/honeybee/config.toml
    #[clap(default_value = "~/.honeybee.toml")]
    pub config_file: String,
    /// Indicates whether colors will be disabled or not.
    #[arg(long)]
    pub no_colors: bool,
    /// Indicates whether honeybee will stop after the first status line.
    #[arg(long)]
    pub run_once: bool,
    /// Maximum number of blocking threads used by honeybee.
    #[arg(long, default_value_t = 5)]
    pub num_threads: usize,
}
