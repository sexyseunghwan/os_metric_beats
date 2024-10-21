use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct IndexPattern {
    pub index_pattern: String
}