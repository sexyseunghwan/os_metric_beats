use crate::common::*;

#[derive(Deserialize, Debug, new)]
pub struct Win32MemRes {
    pub working_set_size: u64,
    pub virtual_size: u64,
}