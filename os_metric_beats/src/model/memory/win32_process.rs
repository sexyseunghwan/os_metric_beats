use crate::common::*;

#[derive(Deserialize, Debug)]
pub struct Win32Process {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ProcessId")]
    pub process_id: u32,
    #[serde(rename = "WorkingSetSize")]
    pub working_set_size: u64,
    #[serde(rename = "VirtualSize")]
    pub virtual_size: u64,
}
