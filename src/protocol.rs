use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smart_default::SmartDefault;

use crate::bar::RenderedWidget;
use crate::output::color::Color;

#[skip_serializing_none]
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Serialize)]
pub struct Header {
    /// The version number (as an integer) of the i3bar protocol you will use.
    #[default = 1]
    version: usize,

    /// Specify the signal (as an integer) that i3bar should send to request that you pause your output.
    /// The default value is SIGSTOP, which will unconditionally stop your process.
    stop_signal: Option<usize>,

    /// Specify to i3bar the signal (as an integer) to send to continue your processing. The default
    /// value (if none is specified) is SIGCONT.
    cont_signal: Option<usize>,

    /// If specified and true i3bar will write an infinite array (same as above) to your stdin.
    #[default(Some(true))]
    click_events: Option<bool>,
}

impl Header {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum BlockWidth {
    Pixels(usize),
    Text(String),
}

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Serialize)]
pub enum Alignment {
    Center,
    Right,
    #[default]
    Left,
}

#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Deserialize, Serialize)]
pub enum Markup {
    #[default]
    None,
    Pango,
}

#[skip_serializing_none]
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Serialize)]
pub struct Block {
    pub full_text: String,
    pub short_text: Option<String>,
    #[default(Some(Default::default()))]
    pub color: Option<Color>,
    #[default(Some(Color::try_from("#000000").unwrap()))]
    pub background: Option<Color>,
    #[default(Some(Color::try_from("#222222").unwrap()))]
    pub border: Option<Color>,
    #[default(Some(1))]
    pub border_top: Option<usize>,
    #[default(Some(1))]
    pub border_right: Option<usize>,
    #[default(Some(1))]
    pub border_bottom: Option<usize>,
    #[default(Some(1))]
    pub border_left: Option<usize>,
    pub min_width: Option<BlockWidth>,
    pub align: Alignment,
    pub urgent: Option<bool>,
    pub name: Option<String>,
    pub instance: Option<String>,
    #[default(Some(true))]
    pub separator: Option<bool>,
    #[default(Some(9))]
    pub separator_block_width: Option<usize>,
    pub markup: Markup,
}

impl Block {
    pub fn new(full_text: String) -> Self {
        Self {
            full_text,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClickEvent;

pub fn init() {
    println!("{}\n[", serde_json::to_string(&Header::new()).unwrap());
}

pub fn status_line(rendered_widgets: &[RenderedWidget]) {
    let v: Vec<_> = rendered_widgets
        .iter()
        .filter_map(|r| {
            if let RenderedWidget::I3Block(b) = r {
                Some(b.clone())
            } else {
                None
            }
        })
        .collect();

    println!("  {}", serde_json::to_string(&v).unwrap());
}

pub fn stop() {
    println!("]");
}
