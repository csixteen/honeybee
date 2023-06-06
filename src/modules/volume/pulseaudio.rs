use super::Volume;
use crate::errors::*;

pub(super) fn get_volume() -> Result<Volume> {
    Err(Error::new("No PulseAudio found"))
}
