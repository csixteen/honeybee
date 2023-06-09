//! Structs and methods to manipulate widgets. A [`Widget`] is a graphical element
//! that is used to render the output of a [`Module`].
//!
//! [`Module`]: crate::modules
use std::fmt::{self, Formatter};

use crate::config::GeneralConfig;
use crate::formatting::{Format, Placeholders};
use crate::output::color::Color;

/// The graphical element that is used to render the
/// output of a module.
#[derive(Clone, Debug, Default)]
pub struct Widget {
    pub name: String,
    pub instance: Option<String>,
    pub state: WidgetState,
    pub content: ContentType,
}

impl Widget {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_name(self, name: String) -> Self {
        Self { name, ..self }
    }

    pub fn with_instance(self, instance: String) -> Self {
        Self {
            instance: Some(instance),
            ..self
        }
    }

    pub fn set_state(&mut self, state: WidgetState) {
        self.state = state;
    }

    pub fn set_format(&mut self, format: Format) {
        if let ContentType::Format(f, _) = &mut self.content {
            *f = format;
        } else {
            self.content = ContentType::Format(format, None);
        }
    }

    pub fn set_placeholders(&mut self, placeholders: Placeholders) {
        if let ContentType::Format(_, p) = &mut self.content {
            *p = Some(placeholders);
        }
    }

    pub fn color(&self, config: &GeneralConfig) -> Color {
        match &self.state {
            WidgetState::Normal => config.color_good.clone(),
            WidgetState::Warning => config.color_degraded.clone(),
            WidgetState::Critical => config.color_bad.clone(),
        }
    }
}

/// The state of the widget will define its color value, which can be
/// either `color_good`, `color_degraded` or `color_bad`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum WidgetState {
    #[default]
    Normal,
    Warning,
    Critical,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ContentType {
    #[default]
    None,
    Text(String),
    Format(Format, Option<Placeholders>),
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ContentType::None => write!(f, ""),
            ContentType::Text(s) => write!(f, "{}", s),
            ContentType::Format(format, p) => {
                // TODO - figure out a way of using full_text and short_text properly
                let (full_text, _) = format.render(p);
                write!(f, "{}", full_text)
            }
        }
    }
}
