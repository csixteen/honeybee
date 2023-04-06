use crate::modules::ModuleConfig;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::output::color::Color;
use crate::protocol::Markup;

#[derive(Clone, Debug, SmartDefault, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    pub general: GeneralConfig,

    #[serde(rename = "module")]
    pub modules: Vec<ModuleConfig>,
}

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    #[default = "i3bar"]
    pub output_format: String,
    #[default = true]
    pub colors: bool,
    pub separator: String,
    #[default(Color::try_from("#333333").unwrap())]
    pub color_separator: Color,
    #[default(Color::try_from("#00FF00").unwrap())]
    pub color_good: Color,
    #[default(Color::try_from("#FFFF00").unwrap())]
    pub color_degraded: Color,
    #[default(Color::try_from("#FF0000").unwrap())]
    pub color_bad: Color,
    pub markup: Markup,
    #[default = 10]
    pub update_interval: u64,
}
