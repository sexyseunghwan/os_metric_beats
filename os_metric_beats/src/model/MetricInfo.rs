use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct MetricInfo {
     pub timestamp: String,
     pub host: String,
     pub system_cpu_usage: f32,
     pub system_disk_usage: f64,
     pub system_memory_usage: f64,
}