use crate::contracts::common::ApiMeta;
use crate::runtime::ContractSnapshot;

pub fn fallback_meta(snapshot: &ContractSnapshot) -> ApiMeta {
    snapshot
        .site
        .as_ref()
        .map(|site| site.meta.clone())
        .unwrap_or(ApiMeta {
            version: "unknown".to_string(),
            generated_at: String::new(),
        })
}
