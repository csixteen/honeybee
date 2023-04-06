use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::errors::*;

const HONEYBEE_CFG: &str = "honeybee/config.toml";

pub fn read_toml_config<T, P>(path: P) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let contents = fs::read_to_string(path).error("Couldn't read config file.")?;
    toml::from_str(&contents).map_err(|e| {
        Error::new(format!(
            "Failed to deserialize TOML config: {}",
            e.message()
        ))
    })
}

/// This will try to find the config file path in the following
/// locations (by order):
///
/// 1. $XDG_CONFIG_HOME/honeybee/config.toml
/// 2. $XDG_DATA_HOME/honeybee/config.toml
/// 3. $HOME/.honeybee.toml
/// 4. $XDG_DATA_DIRS/honeybee/config.toml
/// 5. $XDG_CONFIG_DIRS/honeybee/config.toml
///
/// Reference: https://wiki.archlinux.org/title/XDG_Base_Directory
pub fn get_config_path(file: &str) -> Option<PathBuf> {
    let t = shellexpand::tilde(file);
    let f = Path::new(t.as_ref());
    if f.is_absolute() && f.exists() {
        return Some(f.to_path_buf());
    }

    file_path(dirs::config_dir(), HONEYBEE_CFG)
        .or_else(|| file_path(dirs::data_dir(), HONEYBEE_CFG))
        .or_else(|| file_path(dirs::home_dir(), ".honeybee.toml"))
        .or_else(|| {
            xdg_data_dirs()
                .iter()
                .map(|x| file_path(x.clone(), HONEYBEE_CFG))
                .find(|d| d.is_some())?
        })
        .or_else(|| {
            xdg_config_dirs()
                .iter()
                .map(|x| file_path(x.clone(), HONEYBEE_CFG))
                .find(|d| d.is_some())?
        })
}

fn file_path(base: Option<PathBuf>, ext: &str) -> Option<PathBuf> {
    if let Some(mut d) = base {
        d.push(ext);
        if d.exists() {
            return Some(d);
        }
    }

    None
}

fn xdg_config_dirs() -> Vec<Option<PathBuf>> {
    xdg_dirs("XDG_CONFIG_DIRS", vec![Some(PathBuf::from("/etc/xdg"))])
}

fn xdg_data_dirs() -> Vec<Option<PathBuf>> {
    xdg_dirs(
        "XDG_DATA_DIRS",
        vec![
            Some(PathBuf::from("/usr/local/share")),
            Some(PathBuf::from("/usr/share")),
        ],
    )
}

fn xdg_dirs(env_var: &str, xs: Vec<Option<PathBuf>>) -> Vec<Option<PathBuf>> {
    match env::var_os(env_var) {
        None => xs,
        Some(s) => s
            .into_string()
            .unwrap()
            .split(':')
            .map(|p| dirs_sys::is_absolute_path(OsString::from(p)))
            .collect(),
    }
}
