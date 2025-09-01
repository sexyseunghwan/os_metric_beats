use crate::common::*;

#[derive(Deserialize, Debug, new)]
pub struct OsMemRes {
    pub working_set_size: u64,
    pub virtual_size: u64,
}
