use std::sync::Arc;

use bytes::BytesMut;
use tokio::fs;
use tokio::io::AsyncReadExt;

use super::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    title: String,
    path: String,
    format: Option<Format>,
    format_bad: Option<Format>,
    max_characters: Option<usize>,
}

pub(crate) async fn run(config: Config, bridge: Bridge) -> Result<()> {
    let mut widget = Widget::new().with_instance(config.path.clone());
    let mut timer = bridge.timer().start();
    let format = config.format.unwrap_or("$title: $content".parse()?);
    let format_bad = config.format_bad.unwrap_or("$title: $error".parse()?);
    let max_chars = config.max_characters.unwrap_or(255);

    loop {
        let contents = read_contents(&config.path, max_chars).await;
        let error = match contents {
            Ok(_) => String::new(),
            Err(ref e) => e.to_string(),
        };

        if contents.is_err() {
            widget.set_state(WidgetState::Critical);
            widget.set_format(format_bad.clone());
        } else {
            widget.set_state(WidgetState::Normal);
            widget.set_format(format.clone());
        }

        widget.set_placeholders(map!(
            "$title" => Value::Text(config.title.clone()),
            "$content" => Value::Text(contents.unwrap_or(String::new())),
            "$error" => Value::Text(error),
        ));

        bridge.set_widget(widget.clone()).await?;

        loop {
            tokio::select! {
                _ = timer.tick() => break,
            }
        }
    }
}

async fn read_contents<P>(path: P, max_chars: usize) -> Result<String>
where
    P: AsRef<Path>,
{
    let mut f = fs::File::open(path)
        .await
        .map_err(|e| Error::new("Couldn't open file").with_source(Arc::new(e)))?;
    let mut buf = BytesMut::with_capacity(max_chars * 4);
    let _ = f
        .read_buf(&mut buf)
        .await
        .map_err(|e| Error::new("Couldn't read file").with_source(Arc::new(e)))?;

    String::from_utf8(buf.to_vec())
        .map(|s| s.trim_end().chars().take(max_chars).collect())
        .error("Invalid UTF-8 string")
}
