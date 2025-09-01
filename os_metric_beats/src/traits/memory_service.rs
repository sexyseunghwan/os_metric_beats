use crate::common::*;

use crate::model::memory::os_mem_res::*;

pub trait MemoryService {
    fn get_process_mem_usage(&self) -> Result<OsMemRes, anyhow::Error>;
}
