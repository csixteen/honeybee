use futures::future::FutureExt;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::bridge::Bridge;
use crate::errors::*;
use crate::types::BoxedFuture;

pub mod memory;
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "module")]
#[serde(deny_unknown_fields)]
pub enum ModuleConfig {
    #[allow(non_camel_case_types)]
    memory {
        #[serde(flatten)]
        config: memory::Config,
    },
}

impl ModuleConfig {
    pub fn run(self, bridge: Bridge) -> BoxedFuture<Result<()>> {
        match self {
            ModuleConfig::memory { config } => memory::run(config, bridge).boxed_local(),
        }
    }
}
