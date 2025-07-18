use crate::common::*;

use crate::model::win32_mem_res::*;

pub trait WmiConnService {
    fn get_process_mem_usage(&self) -> Result<Win32MemRes, anyhow::Error>;
}
