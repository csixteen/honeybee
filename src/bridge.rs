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

    pub(crate) fn timer(&self) -> Timer {
        Timer::new(self.update_interval)
    }

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
