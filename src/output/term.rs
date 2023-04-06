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
