use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    pub name: String,
    pub retry_count: u8,
    pub message: serde_json::Value,
}
