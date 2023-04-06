use futures::future::FutureExt;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::bridge::Bridge;
use crate::errors::*;
use crate::types::BoxedFuture;

mod memory;
mod prelude;

#[derive(Clone, Debug, SmartDefault)]
pub struct Module;

impl Module {
    pub fn new() -> Self {
        Default::default()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum ModuleState {
    #[default]
    None,
    Running,
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
