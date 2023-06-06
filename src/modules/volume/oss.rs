use super::Volume;
use crate::errors::*;

pub(super) fn get_volume() -> Result<Volume> {
    Ok(Volume::Muted)
}
