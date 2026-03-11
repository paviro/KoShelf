use std::collections::BTreeMap;

use crate::contracts::calendar::{
    ActivityMonthResponse, ActivityMonthsResponse, CalendarMonthlyStats,
};
use crate::contracts::common::{ApiMeta, ContentTypeFilter};
use crate::domain::meta::fallback_meta;
use crate::runtime::ContractSnapshot;

pub fn activity_months(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
) -> ActivityMonthsResponse {
    snapshot
        .activity_months
        .get(content_type.as_str())
        .cloned()
        .unwrap_or_else(|| empty_activity_months_response(fallback_meta(snapshot), content_type))
}

pub fn activity_month(
    snapshot: &ContractSnapshot,
    content_type: ContentTypeFilter,
    month_key: &str,
) -> ActivityMonthResponse {
    let meta = snapshot
        .activity_months
        .get(content_type.as_str())
        .map(|months| months.meta.clone())
        .unwrap_or_else(|| fallback_meta(snapshot));

    snapshot
        .activity_months_by_key
        .get(content_type.as_str())
        .and_then(|months| months.get(month_key))
        .cloned()
        .unwrap_or_else(|| empty_activity_month_response(meta, content_type))
}

fn empty_activity_months_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthsResponse {
    ActivityMonthsResponse {
        meta,
        content_type,
        months: vec![],
    }
}

fn empty_activity_month_response(
    meta: ApiMeta,
    content_type: ContentTypeFilter,
) -> ActivityMonthResponse {
    ActivityMonthResponse {
        meta,
        content_type,
        events: vec![],
        items: BTreeMap::new(),
        stats: empty_calendar_monthly_stats(),
    }
}

fn empty_calendar_monthly_stats() -> CalendarMonthlyStats {
    CalendarMonthlyStats {
        items_read: 0,
        pages_read: 0,
        time_read: 0,
        days_read_pct: 0,
    }
}
