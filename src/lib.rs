//! `honeybee` is a port of [i3status], written in Rust. It means that it's compatible with the
//! [i3bar protocol], but also generates status line for dzen2,xmobar and lemonbar.
//!
//! [i3status]: https://github.com/i3/i3status/
//! [i3bar protocol]: https://i3wm.org/docs/i3bar-protocol.html

#![cfg_attr(docsrs, feature(doc_cfg))]

#[allow(clippy::large_enum_variant)]
pub mod bar;
pub mod bridge;
pub mod config;
pub mod errors;
pub mod formatting;
pub mod modules;
pub mod output;
pub mod protocol;
pub mod timer;
pub mod types;
pub mod units;
pub mod utils;
pub mod widget;

use clap::Parser;

/// A port of i3status, written in Rust.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about)]
pub struct CliArgs {
    /// If an absolute path is provided, then it shall be used (this also performs
    /// tilde expansion). If it's not an absolute path, or if the file doesn't exist,
    /// then a valid confdiguration will be searched using the following order:
    ///
    /// 1. `$XDG_CONFIG_HOME/honeybee/config.toml`
    ///
    /// 2. `$XDG_DATA_HOME/honeybee/config.toml`
    ///
    /// 3. `$HOME/.honeybee.toml`
    ///
    /// 4. `$XDG_DATA_DIRS/honeybee/config.toml`
    ///
    /// 5. `$XDG_CONFIG_DIRS/honeybee/config.toml`
    #[clap(default_value = "honeybee.toml")]
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

#[macro_export]
macro_rules! map {
    ($( $key:literal => $value:expr ),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut m = ::std::collections::HashMap::new();
        $(
        m.insert($key.into(), $value.into());
        )*
        m
    }};
}

// Reference: https://github.com/greshake/i3status-rust/blob/master/src/blocks.rs#L20
// I modified slightly, to adapt to my naming convention. Also, I want to be able to
// conditionally compile certain modules based on the `target_os` as well.
#[macro_export]
macro_rules! modules {
    {
        $( $(#[cfg($x: ident = $xp : literal)])? $module: ident $(,)? )*
    } => {
        $(
            $(#[cfg($x = $xp)])?
            $(#[cfg_attr(docsrs, doc(cfg($x = $xp)))])?
            pub mod $module;
        )*

        #[derive(Clone, Debug, Deserialize)]
        #[serde(tag = "module")]
        #[serde(deny_unknown_fields)]
        pub enum ModuleConfig {
            $(
                $(#[cfg($x = $xp)])?
                #[allow(non_camel_case_types)]
                $module {
                    #[serde(flatten)]
                    config: $module::Config,
                },
            )*
        }

        impl ModuleConfig {
            pub fn run(self, bridge: Bridge) -> BoxedFuture<Result<()>> {
                match self {
                    $(
                        $(#[cfg($x = $xp)])?
                        Self::$module { config } => {
                            $module::run(config, bridge).boxed_local()
                        }
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! from_str {
    ($t:tt, $e:expr, $msg:expr) => {{
        let e = $e;
        $t::from_str(e).or_error(|| format!("{}", $msg))?
    }};
}
