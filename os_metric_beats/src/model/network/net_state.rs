use crate::common::*;

use crate::model::network::iface_counters::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetState {
    pub ifaces: HashMap<String, IfaceCounters>,
    pub statistics: HashMap<String, IfaceCounters>,
    pub updated_at: String,
}

impl NetState {
    pub fn new(updated_at: String) -> Self {
        Self {
            ifaces: HashMap::new(),
            statistics: HashMap::new(),
            updated_at,
        }
    }

    pub fn add_iface(&mut self, name: String, rx: u64, tx: u64) {
        self.ifaces.insert(name, IfaceCounters::new(rx, tx));
    }

    pub fn get_iface(&self, name: &str) -> Option<&IfaceCounters> {
        self.ifaces.get(name)
    }

    pub fn add_statistics(&mut self, name: String, rx: u64, tx: u64) {
        self.statistics.insert(name, IfaceCounters::new(rx, tx));
    }

    pub fn get_statistics(&self, name: &str) -> Option<&IfaceCounters> {
        self.statistics.get(name)
    }

    pub fn update_timestamp(&mut self, timestamp: String) {
        self.updated_at = timestamp;
    }
}
