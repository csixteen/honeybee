use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dzen2;

impl OutputFormatter for Dzen2 {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let c = widget.color(config);
        format!("^fg({c})")
    }

    fn full_text(&self, config: &GeneralConfig, widget: &Widget) -> String {
        let content = widget.content.to_string();
        let color = self.fg_color(config, &widget);
        format!("{}{}", color, content)
    }

    fn status_line(&self, _rendered_widgets: &[RenderedWidget]) {
        todo!()
    }
}
