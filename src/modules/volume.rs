mod alsa;
#[cfg(feature = "pulseaudio")]
mod pulseaudio;

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
    mixer_idx: u32,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(format!(
        "{}.{}.{}",
        config.device, config.mixer, config.mixer_idx
    ));
    let mut timer = bridge.timer().start();

    loop {
        let vol = match get_volume(&config)? {
            Volume::Muted => {
                widget.set_state(WidgetState::Warning);
                widget.set_format(config.format_muted.clone());
                0_f64
            }
            Volume::Unmuted(v) => {
                widget.set_state(WidgetState::Normal);
                widget.set_format(config.format.clone());
                v
            }
        };

        widget.set_placeholders(map!(
            "$device" => Value::Text(config.device.clone()),
            "$volume" => Value::percentage(vol)
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

#[cfg(not(feature = "pulseaudio"))]
fn get_volume(config: &Config) -> Result<Volume> {
    alsa::get_volume(&config.device, &config.mixer, config.mixer_idx)
}

#[cfg(feature = "pulseaudio")]
fn get_volume(config: &Config) -> Result<Volume> {
    pulseaudio::get_volume()
        .or_else(|_| alsa::get_volume(&config.device, &config.mixer, config.mixer_idx))
}

#[derive(Clone, Debug, PartialEq)]
enum Volume {
    Muted,
    Unmuted(f64),
}
