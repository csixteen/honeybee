//! [xmobar] is a minimalistic, mostly text based, status bar. It was originally designed
//! and implemented by Andrea Rossato to work with [xmonad], but it's actually usable with
//! any window-manager.
//!
//! [xmobar]: https://archives.haskell.org/projects.haskell.org/xmobar/
//! [xmonad]: https://xmonad.org/
use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmoBar;

impl OutputFormatter for XmoBar {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let c = widget.color(config);
        format!("<fc={c}>")
    }

    fn full_text(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let content = widget.content.to_string();
        let color = self.fg_color(config, widget);
        format!("{}{}</fc>", color, content)
    }

    fn status_line(&self, _rendered_widgets: &[RenderedWidget]) {
        todo!()
    }
}
