use crate::common::*;

use crate::model::memory::os_mem_res::*;
use crate::model::memory::win32_process::*;

use crate::traits::memory_service::*;

#[cfg(windows)]
use wmi::*;

#[derive(Debug)]
pub struct WmiConnServiceImpl {
    #[cfg(windows)]
    wmi_conn: WMIConnection,
}

impl WmiConnServiceImpl {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            let com_lib: COMLibrary = Self::init_com_library();
            let wmi_conn: WMIConnection = Self::init_wmi_connection(com_lib);
            WmiConnServiceImpl { wmi_conn }
        }

        #[cfg(not(windows))]
        {
            WmiConnServiceImpl {}
        }
    }

    #[cfg(windows)]
    fn init_com_library() -> COMLibrary {
        COMLibrary::new().unwrap_or_else(|e| {
            error!("[Error][WmiConnServicePub->init_com_library] {:?}", e);
            panic!("[Panic][WmiConnServicePub->init_com_library] {:?}", e);
        })
    }

    #[cfg(windows)]
    fn init_wmi_connection(com_lib: COMLibrary) -> WMIConnection {
        WMIConnection::new(com_lib).unwrap_or_else(|e| {
            error!("[Error][WmiConnServicePub->init_wmi_connection] {:?}", e);
            panic!("[Panic][WmiConnServicePub->init_wmi_connection] {:?}", e);
        })
    }
}

impl MemoryService for WmiConnServiceImpl {
    #[doc = "윈도우 시스템에서 특정 프로세스가 어느정도의 메모리를 사용하는지 확인해주는 함수"]
    fn get_process_mem_usage(&self) -> Result<OsMemRes, anyhow::Error> {
        #[cfg(windows)]
        {
            let target_keywords: [&str; 3] = ["java", "jdk", "elasticsearch"];

            let mut total_working_set_size: u64 = 0;
            let mut total_virtual_size: u64 = 0;

            let query: &str = r#"
                SELECT Name, ProcessId, WorkingSetSize, VirtualSize
                FROM Win32_Process
                WHERE Name LIKE "%java%" OR Name LIKE "%jdk%" OR Name LIKE "%elasticsearch%""#;

            let results: Vec<Win32Process> = self.wmi_conn.raw_query(query)?;

            for proc in results {
                let name_lower: String = proc.name.to_lowercase();

                if target_keywords.iter().any(|kw| name_lower.contains(kw)) {
                    total_working_set_size += proc.working_set_size;
                    total_virtual_size += proc.virtual_size;
                }
            }

            Ok(OsMemRes::new(total_working_set_size, total_virtual_size))
        }

        #[cfg(not(windows))]
        {
            Ok(OsMemRes::new(0, 0))
        }
    }
}
