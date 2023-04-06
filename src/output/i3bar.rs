use super::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I3Bar;

impl OutputFormatter for I3Bar {
    fn fg_color(&self, config: &GeneralConfig, widget: &Widget) -> String {
        widget.color(config).to_string()
    }

    fn full_text(&self, _config: &GeneralConfig, widget: &Widget) -> String {
        widget.content.to_string()
    }

    fn render_widget(&self, config: &GeneralConfig, widget: Widget) -> RenderedWidget {
        let block = Block {
            full_text: self.full_text(config, &widget),
            color: Some(widget.color(config)),
            ..Default::default()
        };
        RenderedWidget::I3Block(block)
    }

    fn init(&self) {
        protocol::init();
    }

    fn status_line(&self, rendered_widgets: &[RenderedWidget]) {
        protocol::status_line(rendered_widgets);
    }

    fn stop(&self) {
        protocol::stop();
    }
}
