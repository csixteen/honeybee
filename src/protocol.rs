//! i3bar protocol. For full reference, check the main [documentation].
//!
//! The main structures of this module are [`Header`], [`Block`] and [`ClickEvent`]. This is only
//! relevant if you choose [`i3bar`] as the `output_format`.
//!
//! [documentation]: https://i3wm.org/docs/i3bar-protocol.html

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smart_default::SmartDefault;

use crate::bar::RenderedWidget;
use crate::output::color::Color;

/// The first message in the protocol is a header block.
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
    /// The width of the text will determine the block width
    /// in pixels.
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

/// Represents the contents of the status line for a particular module.
#[skip_serializing_none]
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq, Serialize)]
pub struct Block {
    /// This is the only required key by the protocol. If it's an empty string,
    /// then the block will be skipped.
    pub full_text: String,
    /// If provided, the `short_text` will be used when the status line needs to be shortened
    /// (because it uses more space than your screen provides).
    pub short_text: Option<String>,
    /// The color used to display the contents of `full_text` or `short_text`.
    #[default(Some(Default::default()))]
    pub color: Option<Color>,
    /// Overrides the background color for this particular block.
    #[default(Some(Color::try_from("#000000").unwrap()))]
    pub background: Option<Color>,
    /// Overrides the border color for this particular block.
    #[default(Some(Color::try_from("#222222").unwrap()))]
    pub border: Option<Color>,
    #[default(Some(1))]
    /// Defines the width (in pixels) of the top border of this block.
    pub border_top: Option<usize>,
    /// Defines the width (in pixels) of the right border of this block.
    #[default(Some(1))]
    pub border_right: Option<usize>,
    /// Defines the width (in pixels) of the bottom border of this block.
    #[default(Some(1))]
    pub border_bottom: Option<usize>,
    /// Defines the width (in pixels) of the left border of this block.
    #[default(Some(1))]
    pub border_left: Option<usize>,
    /// The minimum width, in pixels, of this block. If `full_text` takes less
    /// space than the `min_width`, then the block will be padded to the left
    /// and/or right, depending on the `alignment`.
    pub min_width: Option<BlockWidth>,
    /// Aligns the text on the center, left or right, when the `min_width` is not
    /// reached.
    pub align: Alignment,
    /// Specifies whether the current value is urgent (e.g. no more disk space).
    pub urgent: Option<bool>,
    /// Unique name of this block, used to identify it in scripts when processing
    /// the output.
    pub name: Option<String>,
    /// In case there are multiple instances of a block (e.g. multiple disk space
    /// blocks for multiple mount points).
    pub instance: Option<String>,
    /// Indicates whether a separator line should be drawn after this block.
    #[default(Some(true))]
    pub separator: Option<bool>,
    /// Amount of pixels to leave blank after the block. Unless `separator` is disabled,
    /// a separator line will be drawn in the middle of this gap.
    #[default(Some(9))]
    pub separator_block_width: Option<usize>,
    /// Indicates how the block should be parsed.
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
