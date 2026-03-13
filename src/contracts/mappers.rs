use std::collections::BTreeMap;
use std::collections::HashMap;

use super::common::ApiMeta;
use super::site::{SiteCapabilities, SiteResponse};
use crate::models::ContentType;

pub fn build_meta(version: impl Into<String>, generated_at: impl Into<String>) -> ApiMeta {
    ApiMeta {
        version: version.into(),
        generated_at: generated_at.into(),
    }
}

pub fn map_site_response(
    meta: ApiMeta,
    title: impl Into<String>,
    language: impl Into<String>,
    capabilities: SiteCapabilities,
) -> SiteResponse {
    SiteResponse {
        meta,
        title: title.into(),
        language: language.into(),
        capabilities,
    }
}

pub fn years_for_content_type(
    year_month_items: &HashMap<i32, BTreeMap<String, Vec<crate::models::RecapItem>>>,
    target: ContentType,
) -> Vec<i32> {
    let mut years: Vec<i32> = year_month_items
        .iter()
        .filter_map(|(year, months)| {
            let has_target = months
                .values()
                .flatten()
                .any(|item| item.content_type == Some(target));
            if has_target { Some(*year) } else { None }
        })
        .collect();
    years.sort_by(|a, b| b.cmp(a));
    years.dedup();
    years
}
