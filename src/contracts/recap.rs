use serde::{Deserialize, Serialize};

use super::common::{ApiMeta, Scope, Scoped};
use super::library::LibraryContentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapIndexScope {
    pub available_years: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapIndexResponse {
    pub meta: ApiMeta,
    pub scopes: Scoped<RecapIndexScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapIndexScopedResponse {
    pub meta: ApiMeta,
    pub available_years: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapSummaryResponse {
    pub total_books: usize,
    pub total_time_seconds: i64,
    pub total_time_days: i64,
    pub total_time_hours: i64,
    pub longest_session_hours: i64,
    pub longest_session_minutes: i64,
    pub average_session_hours: i64,
    pub average_session_minutes: i64,
    pub active_days: usize,
    pub active_days_percentage: f64,
    pub longest_streak: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_month_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_month_time_display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapItemResponse {
    pub title: String,
    pub authors: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub reading_time: i64,
    pub session_count: i64,
    pub pages_read: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<LibraryContentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapMonthResponse {
    pub month_key: String,
    pub month_label: String,
    pub books_finished: usize,
    pub read_time: i64,
    pub items: Vec<RecapItemResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapShareAssets {
    pub story_url: String,
    pub square_url: String,
    pub banner_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapYearScope {
    pub summary: RecapSummaryResponse,
    pub months: Vec<RecapMonthResponse>,
    pub items: Vec<RecapItemResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_assets: Option<RecapShareAssets>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapYearResponse {
    pub meta: ApiMeta,
    pub year: i32,
    pub scopes: Scoped<RecapYearScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecapYearScopedResponse {
    pub meta: ApiMeta,
    pub year: i32,
    #[serde(flatten)]
    pub payload: RecapYearScope,
}

impl RecapIndexResponse {
    pub fn scoped(&self, scope: Scope) -> RecapIndexScopedResponse {
        let selected = self.scopes.select(scope).clone();
        RecapIndexScopedResponse {
            meta: self.meta.clone(),
            available_years: selected.available_years,
            latest_year: selected.latest_year,
        }
    }
}

impl RecapYearResponse {
    pub fn scoped(&self, scope: Scope) -> RecapYearScopedResponse {
        RecapYearScopedResponse {
            meta: self.meta.clone(),
            year: self.year,
            payload: self.scopes.select(scope).clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn project_scope_payload(value: Value, scope: Scope) -> Value {
        let mut root = match value {
            Value::Object(map) => map,
            other => return other,
        };

        let Some(Value::Object(scopes)) = root.remove("scopes") else {
            return Value::Object(root);
        };

        let selected = scopes
            .get(scope.as_str())
            .cloned()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(selected_scope) = selected {
            for (key, value) in selected_scope {
                root.insert(key, value);
            }
        }

        Value::Object(root)
    }

    fn sample_meta() -> ApiMeta {
        ApiMeta {
            version: "test".to_string(),
            generated_at: "2026-03-03T00:00:00+00:00".to_string(),
        }
    }

    fn sample_index_scope(year: i32) -> RecapIndexScope {
        RecapIndexScope {
            available_years: vec![year],
            latest_year: Some(year),
        }
    }

    fn sample_year_scope(tag: &str) -> RecapYearScope {
        RecapYearScope {
            summary: RecapSummaryResponse {
                total_books: 1,
                total_time_seconds: 1,
                total_time_days: 0,
                total_time_hours: 1,
                longest_session_hours: 0,
                longest_session_minutes: 30,
                average_session_hours: 0,
                average_session_minutes: 15,
                active_days: 1,
                active_days_percentage: 1.0,
                longest_streak: 1,
                best_month_name: Some(tag.to_string()),
                best_month_time_display: Some("1h".to_string()),
            },
            months: vec![RecapMonthResponse {
                month_key: "2026-03".to_string(),
                month_label: "March".to_string(),
                books_finished: 1,
                read_time: 1,
                items: vec![RecapItemResponse {
                    title: format!("{} title", tag),
                    authors: vec!["Author".to_string()],
                    start_date: "2026-03-01".to_string(),
                    end_date: "2026-03-02".to_string(),
                    reading_time: 1,
                    session_count: 1,
                    pages_read: 1,
                    rating: Some(5),
                    review_note: Some("note".to_string()),
                    series: None,
                    item_path: Some("/books/id/".to_string()),
                    item_cover: Some("/assets/covers/id.webp".to_string()),
                    content_type: Some(LibraryContentType::Book),
                }],
            }],
            items: vec![],
            share_assets: Some(RecapShareAssets {
                story_url: "/assets/recap/story.webp".to_string(),
                square_url: "/assets/recap/square.webp".to_string(),
                banner_url: "/assets/recap/banner.webp".to_string(),
            }),
        }
    }

    #[test]
    fn recap_index_scoped_projects_selected_scope() {
        let response = RecapIndexResponse {
            meta: sample_meta(),
            scopes: Scoped {
                all: sample_index_scope(2026),
                books: sample_index_scope(2025),
                comics: sample_index_scope(2024),
            },
        };

        let scoped = response.scoped(Scope::Comics);
        assert_eq!(scoped.available_years, vec![2024]);
        assert_eq!(scoped.latest_year, Some(2024));

        let json = serde_json::to_value(scoped).expect("scoped recap index should serialize");
        assert!(json.get("scopes").is_none());
        assert!(json.get("available_years").is_some());
    }

    #[test]
    fn recap_year_scoped_flattens_selected_scope() {
        let response = RecapYearResponse {
            meta: sample_meta(),
            year: 2026,
            scopes: Scoped {
                all: sample_year_scope("all"),
                books: sample_year_scope("books"),
                comics: sample_year_scope("comics"),
            },
        };

        let scoped = response.scoped(Scope::Books);
        assert_eq!(
            scoped.payload.summary.best_month_name.as_deref(),
            Some("books")
        );

        let json = serde_json::to_value(scoped).expect("scoped recap year should serialize");
        assert!(json.get("scopes").is_none());
        assert!(json.get("summary").is_some());
        assert!(json.get("months").is_some());
    }

    #[test]
    fn recap_scoped_projection_matches_legacy_scope_projection() {
        let response = RecapYearResponse {
            meta: sample_meta(),
            year: 2026,
            scopes: Scoped {
                all: sample_year_scope("all"),
                books: sample_year_scope("books"),
                comics: sample_year_scope("comics"),
            },
        };

        let legacy = project_scope_payload(
            serde_json::to_value(&response).expect("legacy payload should serialize"),
            Scope::Comics,
        );
        let typed = serde_json::to_value(response.scoped(Scope::Comics))
            .expect("typed scoped payload should serialize");

        assert_eq!(typed, legacy);
    }
}
