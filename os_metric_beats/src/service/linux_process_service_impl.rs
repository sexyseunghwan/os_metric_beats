use crate::common::*;

use crate::model::memory::os_mem_res::*;

use crate::traits::memory_service::*;

#[derive(Debug)]
pub struct LinuxProcessServiceImpl;

impl LinuxProcessServiceImpl {
    pub fn new() -> Self {
        LinuxProcessServiceImpl
    }

    #[doc = "프로세스의 메모리 사용량을 계산하는 함수 - 리눅스 버전"]
    fn get_process_memory_usage_linux(
        &self,
        keywords: &[&str],
    ) -> Result<OsMemRes, anyhow::Error> {
        let mut sys: System = System::new_all();
        sys.refresh_all();
        
        
        let mut total_rss_byte: u64 = 0;
        let mut total_vms_byte: u64 = 0;

        for (_pid, proc_) in sys.processes() {
            let name_lower: String = proc_.name().to_lowercase();

            if keywords.iter().any(|kw| name_lower.contains(&kw.to_lowercase())) {
                /* sysinfo: memory()와 virtual_memory()는 KiB 단위 */ 
                total_rss_byte += proc_.memory();
                total_vms_byte += proc_.virtual_memory();
            }
        }

        Ok(OsMemRes::new(total_rss_byte, total_vms_byte))
    }
}

impl MemoryService for LinuxProcessServiceImpl {
    fn get_process_mem_usage(&self) -> Result<OsMemRes, anyhow::Error> {
        let target_keywords: [&str; 3] = ["java", "jdk", "elasticsearch"];

        match self.get_process_memory_usage_linux(&target_keywords) {
            Ok(os_mem_res) => Ok(os_mem_res),
            Err(e) => {
                error!("Failed to get Linux process memory usage: {:?}", e);
                Ok(OsMemRes::new(0, 0))
            }
        }
    }
}
