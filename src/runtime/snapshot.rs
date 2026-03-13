//! Typed in-memory representation of the generated transport contracts.
//!
//! During serve mode the snapshot holds the pre-computed `/api/site` response.
//! All other API responses are computed on demand from domain services.

use crate::contracts::site::SiteResponse;

#[derive(Debug, Clone, Default)]
pub struct ContractSnapshot {
    pub site: Option<SiteResponse>,
}

impl ContractSnapshot {
    pub fn generated_at(&self) -> Option<&str> {
        self.site
            .as_ref()
            .map(|site| site.meta.generated_at.as_str())
    }
}
