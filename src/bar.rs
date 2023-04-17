//! This is the main container of all the running modules.
use std::sync::Arc;

use futures::stream::{FuturesUnordered, StreamExt};
use smart_default::SmartDefault;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::bridge::{Bridge, Request};
use crate::config::GeneralConfig;
use crate::errors::*;
use crate::modules::ModuleConfig;
use crate::output::{output_formatter, OutputFormatter};
use crate::protocol::Block;
use crate::types::BoxedFuture;

#[derive(Debug)]
pub struct Bar {
    general_config: GeneralConfig,
    output_format: Arc<dyn OutputFormatter>,
    update_sender: Sender<Request>,
    update_receiver: Receiver<Request>,
    modules: Vec<(&'static str, usize)>,
    running_modules: FuturesUnordered<BoxedFuture<Result<()>>>,
    rendered_widgets: Vec<RenderedWidget>,
}

impl Bar {
    pub fn new(general_config: GeneralConfig) -> Self {
        let (update_sender, update_receiver) = mpsc::channel(64);
        let output_format = output_formatter(&general_config.output_format);

        Bar {
            general_config,
            update_sender,
            update_receiver,
            output_format,
            modules: Vec::new(),
            running_modules: FuturesUnordered::new(),
            rendered_widgets: Vec::new(),
        }
    }

    pub async fn add_module(&mut self, config: ModuleConfig) -> Result<()> {
        let id = self.running_modules.len();
        let bridge = Bridge::new(
            id,
            self.update_sender.clone(),
            self.general_config.update_interval,
        );
        self.modules.push((config.name(), id));
        self.running_modules.push(Box::pin(config.run(bridge)));
        self.rendered_widgets.push(RenderedWidget::default());

        Ok(())
    }

    fn process_request(&mut self, request: Request) {
        match request {
            Request::SetWidget { id, widget } => {
                self.rendered_widgets[id] = self.output_format.render_widget(
                    &self.general_config,
                    widget.with_name(self.modules[id].0.to_owned()),
                );
            }
        }
    }

    async fn handler(&mut self) -> Result<()> {
        tokio::select! {
            Some(res) = self.running_modules.next() => {
                res
            },
            Some(req) = self.update_receiver.recv() => {
                self.process_request(req);
                self.output_format.status_line(&self.rendered_widgets);
                Ok(())
            }
        }
    }

    pub async fn run(&mut self, run_once: bool) -> Result<()> {
        self.output_format.init();

        loop {
            if let Err(e) = self.handler().await {
                println!("{e}");
                // TODO - handler error
            }

            if run_once {
                self.output_format.stop();
                return Ok(());
            }
        }
    }
}

/// A rendered widget can be simply a string or an i3 Block.
#[derive(Clone, Debug, SmartDefault, Eq, PartialEq)]
pub enum RenderedWidget {
    #[default]
    None,
    Text(String),
    I3Block(Block),
}
