use crate::contracts::common::ApiMeta;
use crate::runtime::ContractSnapshot;

pub fn fallback_meta(snapshot: &ContractSnapshot) -> ApiMeta {
    snapshot
        .site
        .as_ref()
        .map(|site| site.meta.clone())
        .or_else(|| snapshot.items.as_ref().map(|items| items.meta.clone()))
        .or_else(|| {
            snapshot
                .activity_weeks
                .values()
                .next()
                .map(|weeks| weeks.meta.clone())
        })
        .or_else(|| {
            snapshot
                .activity_months
                .values()
                .next()
                .map(|months| months.meta.clone())
        })
        .or_else(|| {
            snapshot
                .completion_years
                .values()
                .next()
                .map(|years| years.meta.clone())
        })
        .unwrap_or(ApiMeta {
            version: "unknown".to_string(),
            generated_at: "".to_string(),
        })
}
