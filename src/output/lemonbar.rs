//! [lemonbar] is a lightweight bar based entirely on XCB. It has full UTF-8 support
//! and is EWMH compliant.
//!
//! [lemonbar]: https://github.com/LemonBoy/bar
use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LemonBar;

impl OutputFormatter for LemonBar {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let c = widget.color(config);
        format!("%%{{F{c}}}")
    }

    fn full_text(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let content = widget.content.to_string();
        let color = self.fg_color(config, widget);
        format!("{}{}", color, content)
    }

    fn status_line(&self, _rendered_widgets: &[RenderedWidget]) {
        todo!()
    }
}
