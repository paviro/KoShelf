use serde::{Deserialize, Serialize};

use super::common::ApiMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalesResponse {
    pub meta: ApiMeta,
    pub language: String,
    pub resources: Vec<String>,
}
