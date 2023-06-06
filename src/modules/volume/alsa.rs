use std::sync::Arc;

use super::Volume;
use crate::errors::*;

pub(super) fn get_volume(device: &str, mixer: &str, mixer_idx: u32) -> Result<Volume> {
    let m = alsa::Mixer::new(device, true).map_err(|e| {
        Error::new(format!("Cannot open mixer for device {device}")).with_source(Arc::new(e))
    })?;
    let selem = m
        .find_selem(&alsa::mixer::SelemId::new(mixer, mixer_idx))
        .or_error(|| format!("ALSA: cannot find mixer {mixer} (index {mixer_idx}"))?;

    let channel = Channel::new(mixer, &selem);
    channel.get_volume()
}

#[derive(Clone)]
enum Channel<'a> {
    Capture(&'a alsa::mixer::Selem<'a>),
    Playback(&'a alsa::mixer::Selem<'a>),
}

impl<'a> Channel<'a> {
    fn new(mixer: &str, selem: &'a alsa::mixer::Selem) -> Self {
        if mixer == "capture" {
            Self::Capture(selem)
        } else {
            Self::Playback(selem)
        }
    }

    fn get_range(&self) -> (i64, i64) {
        match self {
            Channel::Capture(selem) => selem.get_capture_volume_range(),
            Channel::Playback(selem) => selem.get_playback_volume_range(),
        }
    }

    fn has_switch(&self) -> bool {
        match self {
            Channel::Capture(selem) => selem.has_capture_switch(),
            Channel::Playback(selem) => selem.has_playback_switch(),
        }
    }

    fn get_switch(&self) -> Result<i32> {
        match self {
            Channel::Capture(selem) => {
                selem.get_capture_switch(alsa::mixer::SelemChannelId::FrontLeft)
            }
            Channel::Playback(selem) => {
                selem.get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft)
            }
        }
        .map_err(|e| Error::new("Alsa: channel switch").with_source(Arc::new(e)))
    }

    fn get_volume(&self) -> Result<Volume> {
        let (min, max) = self.get_range();
        let vol = match self {
            Channel::Capture(selem) => {
                selem.get_capture_volume(alsa::mixer::SelemChannelId::FrontLeft)
            }
            Channel::Playback(selem) => {
                selem.get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            }
        }
        .map_err(|e| Error::new("ALSA: cannot get playback volume").with_source(Arc::new(e)))?;

        let avgf = (((vol - min) as f64) / ((max - min) as f64)) * 100.;

        if self.has_switch() && self.get_switch()? == 0 {
            Ok(Volume::Muted)
        } else {
            Ok(Volume::Unmuted(avgf.round()))
        }
    }
}
