use crate::common::*;

#[doc = "Function to globally initialize the 'ELASTIC_SERVER_INFO' variable"]
pub static ELASTIC_SERVER_INFO: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("ELASTIC_SERVER_INFO")
        .expect("[ENV file read Error] 'ELASTIC_SERVER_INFO' must be set")
});

#[doc = "Function to globally initialize the 'SYSTEM_INFO' variable"]
pub static SYSTEM_INFO: once_lazy<String> = once_lazy::new(|| {
    dotenv().ok();
    env::var("SYSTEM_INFO").expect("[ENV file read Error] 'SYSTEM_INFO' must be set")
});
