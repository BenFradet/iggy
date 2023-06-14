use crate::topic::Topic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stream {
    pub id: u32,
    pub name: String,
    pub topics_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamDetails {
    pub id: u32,
    pub name: String,
    pub topics_count: u32,
    pub topics: Vec<Topic>,
}
