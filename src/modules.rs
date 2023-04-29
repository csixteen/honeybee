use futures::future::FutureExt;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::bridge::Bridge;
use crate::errors::*;
use crate::types::BoxedFuture;

mod prelude;

#[derive(Clone, Debug, SmartDefault)]
pub struct Module;

impl Module {
    pub fn new() -> Self {
        Default::default()
    }
}

/// The state of a module.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum ModuleState {
    #[default]
    None,
    /// State of a healthy running module
    Running,
    /// If there was an error when trying to update.
    Error,
}

crate::modules!(
    battery,
    disk,
    load_avg,
    #[cfg(target_os = "linux")]
    memory,
    path_exists,
    run_watch,
    time
);
