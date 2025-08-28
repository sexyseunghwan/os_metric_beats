use crate::common::*;
use crate::model::win32_mem_res::*;
use crate::traits::wmi_conn_service::*;

#[derive(Debug)]
pub struct LinuxProcessServiceImpl;

impl LinuxProcessServiceImpl {
    pub fn new() -> Self {
        LinuxProcessServiceImpl
    }

    fn get_process_memory_usage_linux(
        &self,
        keywords: &[&str],
    ) -> Result<(u64, u64), anyhow::Error> {
        use std::fs;
        use std::path::Path;

        let mut total_rss = 0u64;
        let mut total_vss = 0u64;

        let proc_dir = Path::new("/proc");
        if let Ok(entries) = fs::read_dir(proc_dir) {
            for entry in entries.flatten() {
                if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                    let status_path = format!("/proc/{}/status", pid);
                    let cmdline_path = format!("/proc/{}/cmdline", pid);

                    if let (Ok(status_content), Ok(cmdline_content)) = (
                        fs::read_to_string(&status_path),
                        fs::read_to_string(&cmdline_path),
                    ) {
                        let cmdline_lower = cmdline_content.replace('\0', " ").to_lowercase();
                        let matches_keyword = keywords.iter().any(|kw| cmdline_lower.contains(kw));

                        if matches_keyword {
                            let mut rss_kb = 0u64;
                            let mut vss_kb = 0u64;

                            for line in status_content.lines() {
                                if line.starts_with("VmRSS:") {
                                    if let Some(value) = line.split_whitespace().nth(1) {
                                        rss_kb = value.parse().unwrap_or(0);
                                    }
                                } else if line.starts_with("VmSize:") {
                                    if let Some(value) = line.split_whitespace().nth(1) {
                                        vss_kb = value.parse().unwrap_or(0);
                                    }
                                }
                            }

                            total_rss += rss_kb * 1024;
                            total_vss += vss_kb * 1024;
                        }
                    }
                }
            }
        }

        Ok((total_rss, total_vss))
    }
}

impl WmiConnService for LinuxProcessServiceImpl {
    fn get_process_mem_usage(&self) -> Result<Win32MemRes, anyhow::Error> {
        let target_keywords: [&str; 3] = ["java", "jdk", "elasticsearch"];

        match self.get_process_memory_usage_linux(&target_keywords) {
            Ok((total_rss, total_vss)) => Ok(Win32MemRes::new(total_rss, total_vss)),
            Err(e) => {
                error!("Failed to get Linux process memory usage: {:?}", e);
                Ok(Win32MemRes::new(0, 0))
            }
        }
    }
}
