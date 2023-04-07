//! The `output_format` configuration defines the format strings honeybee will use
//! in its outputs.
//!
//! Supported output formats are `i3bar`, `dzen2`, `xmobar`, `lemonbar` and `term`.

use std::fmt::Debug;
use std::sync::Arc;

use crate::bar::RenderedWidget;
use crate::config::GeneralConfig;
use crate::widget::Widget;

pub mod color;
pub mod dzen2;
pub mod i3bar;
pub mod lemonbar;
mod prelude;
pub mod term;
pub mod xmobar;

pub trait OutputFormatter: Debug {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String;
    fn full_text(&self, config: &GeneralConfig, widget: &Widget) -> String;
    fn render_widget(&self, config: &GeneralConfig, widget: Widget) -> RenderedWidget {
        RenderedWidget::Text(self.full_text(config, &widget))
    }
    fn init(&self) {}
    fn status_line(&self, rendered_widgets: &[RenderedWidget]);
    fn stop(&self) {}
}

pub fn output_formatter(o: &str) -> Arc<dyn OutputFormatter> {
    match o {
        "term" => Arc::new(term::Term),
        "lemonbar" => Arc::new(lemonbar::LemonBar),
        "i3bar" => Arc::new(i3bar::I3Bar),
        "xmobar" => Arc::new(xmobar::XmoBar),
        "dzen2" => Arc::new(dzen2::Dzen2),
        _ => panic!("Unknown output format {o}"),
    }
}
