use crate::common::*;

#[doc = "env 파일을 읽고 문제가 생길경우에 문제를 로깅해주는 함수"]
fn get_env_var_with_logging(var_name: &str) -> String {
    match env::var(var_name) {
        Ok(value) => {
            info!("Successfully loaded environment variable: {}", var_name);
            value
        }
        Err(env::VarError::NotPresent) => {
            error!("[ERROR][get_env_var_with_logging] Environment variable '{}' is not set. Please check your .env file or environment configuration.", var_name);
            panic!(
                "[ERROR][get_env_var_with_logging] Environment variable '{}' must be set",
                var_name
            );
        }
        Err(env::VarError::NotUnicode(_)) => {
            error!("[ERROR][get_env_var_with_logging] Environment variable '{}' contains invalid Unicode characters.", var_name);
            panic!("[ERROR][get_env_var_with_logging] Environment variable '{}' contains invalid Unicode", var_name);
        }
    }
}

#[doc = "Function to globally initialize the 'ELASTIC_SERVER_INFO' variable"]
pub static ELASTIC_SERVER_INFO: once_lazy<String> =
    once_lazy::new(|| get_env_var_with_logging("ELASTIC_SERVER_INFO"));

#[doc = "Function to globally initialize the 'SYSTEM_INFO' variable"]
pub static SYSTEM_INFO: once_lazy<String> =
    once_lazy::new(|| get_env_var_with_logging("SYSTEM_INFO"));

#[doc = "Function to globally initialize the 'LINUX_CONFIG_INFO' variable"]
pub static LINUX_CONFIG_INFO: once_lazy<String> =
    once_lazy::new(|| get_env_var_with_logging("LINUX_CONFIG_INFO"));

#[doc = "Function to globally initialize the 'NETWORK_NET_INFO_JSON' variable"]
pub static NETWORK_NET_INFO_JSON: once_lazy<String> =
    once_lazy::new(|| get_env_var_with_logging("NETWORK_NET_INFO_JSON"));

#[doc = "Function to globally initialize the 'NETWORK_PACKET_INFO_JSON' variable"]
pub static NETWORK_PACKET_INFO_JSON: once_lazy<String> =
    once_lazy::new(|| get_env_var_with_logging("NETWORK_PACKET_INFO_JSON"));
