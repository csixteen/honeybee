//! Establishes the communication between individual modules and the [`Bar`].
//! Each module has its own bridge.
//!
//! [`Bar`]: crate::bar

use tokio::sync::mpsc::Sender;

use crate::errors::*;
use crate::timer::Timer;
use crate::widget::Widget;

#[derive(Clone, Debug)]
pub struct Bridge {
    id: usize,
    update_sender: Sender<Request>,
    update_interval: u64,
}

impl Bridge {
    pub fn new(id: usize, update_sender: Sender<Request>, update_interval: u64) -> Self {
        Bridge {
            id,
            update_sender,
            update_interval,
        }
    }

    /// Builds a new timer to be used by the corresponding module.
    pub(crate) fn timer(&self) -> Timer {
        Timer::new(self.update_interval)
    }

    /// Tells the [`Bar`] to set the widget for the corresponding module.
    ///
    /// [`Bar`]: crate::bar
    pub(crate) async fn set_widget(&self, widget: Widget) -> Result<()> {
        self.update_sender
            .send(Request::SetWidget {
                id: self.id,
                widget,
            })
            .await
            .map_err(|e| Error::new(e.to_string()))
    }
}

#[derive(Clone, Debug)]
pub enum Request {
    SetWidget { id: usize, widget: Widget },
}
