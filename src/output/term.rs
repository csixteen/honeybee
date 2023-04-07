//! Use ANSI Escape sequences to produce a terminal-output as close as possible to the
//! graphical outputs. This makes debugging your config file a little bit easier because
//! the terminal-output becomes much more readable.
use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Term;

impl OutputFormatter for Term {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let c = widget.color(config);
        format!("\\033[3{c};1m")
    }

    fn full_text(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let content = widget.content.to_string();
        let color = self.fg_color(config, widget);
        format!("{}{}\\033[0m", color, content)
    }

    fn init(&self) {
        todo!()
    }

    fn status_line(&self, _rendered_widgets: &[RenderedWidget]) {
        todo!()
    }
}
