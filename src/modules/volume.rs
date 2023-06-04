use super::prelude::*;

#[derive(Clone, Debug, SmartDefault, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default("default")]
    device: String,
    #[default(Format::new().with_default("V: $volume"))]
    format: Format,
    #[default(Format::new().with_default("V: muted ($volume)"))]
    format_muted: Format,
    #[default("Master")]
    mixer: String,
    #[default(0)]
    mixer_idx: i32,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let widget = Widget::new().with_instance(format!(
        "{}.{}.{}",
        config.device, config.mixer, config.mixer_idx
    ));
    let mut timer = bridge.timer().start();

    loop {
        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}
