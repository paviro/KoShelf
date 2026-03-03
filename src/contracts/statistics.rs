use serde::{Deserialize, Serialize};

use super::common::{ApiMeta, Scope, Scoped};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableWeek {
    pub week_key: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsOverview {
    pub total_read_time: i64,
    pub total_page_reads: i64,
    pub longest_read_time_in_day: i64,
    pub most_pages_in_day: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_session_duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_session_duration: Option<i64>,
    pub total_completions: i64,
    pub books_completed: i64,
    pub most_completions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsStreaks {
    pub longest: crate::models::StreakInfo,
    pub current: crate::models::StreakInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsHeatmapConfig {
    pub max_scale_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsIndexScope {
    pub overview: StatisticsOverview,
    pub streaks: StatisticsStreaks,
    pub heatmap_config: StatisticsHeatmapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsIndexResponse {
    pub meta: ApiMeta,
    pub available_years: Vec<i32>,
    pub available_weeks: Vec<AvailableWeek>,
    pub scopes: Scoped<StatisticsIndexScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsIndexScopedResponse {
    pub meta: ApiMeta,
    pub available_years: Vec<i32>,
    pub available_weeks: Vec<AvailableWeek>,
    pub overview: StatisticsOverview,
    pub streaks: StatisticsStreaks,
    pub heatmap_config: StatisticsHeatmapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsWeekResponse {
    pub meta: ApiMeta,
    pub week_key: String,
    pub scopes: Scoped<crate::models::WeeklyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsWeekScopedResponse {
    pub meta: ApiMeta,
    pub week_key: String,
    #[serde(flatten)]
    pub stats: crate::models::WeeklyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlySummary {
    pub completed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregate {
    pub month_key: String,
    pub read_time: i64,
    pub pages_read: i64,
    pub active_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsYearScope {
    pub summary: YearlySummary,
    pub daily_activity: Vec<crate::models::DailyStats>,
    pub monthly_aggregates: Vec<MonthlyAggregate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<StatisticsHeatmapConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsYearResponse {
    pub meta: ApiMeta,
    pub year: i32,
    pub scopes: Scoped<StatisticsYearScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsYearScopedResponse {
    pub meta: ApiMeta,
    pub year: i32,
    #[serde(flatten)]
    pub payload: StatisticsYearScope,
}

impl StatisticsIndexResponse {
    pub fn scoped(&self, scope: Scope) -> StatisticsIndexScopedResponse {
        let selected = self.scopes.select(scope).clone();
        StatisticsIndexScopedResponse {
            meta: self.meta.clone(),
            available_years: self.available_years.clone(),
            available_weeks: self.available_weeks.clone(),
            overview: selected.overview,
            streaks: selected.streaks,
            heatmap_config: selected.heatmap_config,
        }
    }
}

impl StatisticsWeekResponse {
    pub fn scoped(&self, scope: Scope) -> StatisticsWeekScopedResponse {
        StatisticsWeekScopedResponse {
            meta: self.meta.clone(),
            week_key: self.week_key.clone(),
            stats: self.scopes.select(scope).clone(),
        }
    }
}

impl StatisticsYearResponse {
    pub fn scoped(&self, scope: Scope) -> StatisticsYearScopedResponse {
        StatisticsYearScopedResponse {
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

    fn sample_week(read_time: i64) -> crate::models::WeeklyStats {
        crate::models::WeeklyStats {
            start_date: "2026-03-02".to_string(),
            end_date: "2026-03-08".to_string(),
            read_time,
            pages_read: read_time,
            avg_pages_per_day: 1.0,
            avg_read_time_per_day: 2.0,
            longest_session_duration: Some(3),
            average_session_duration: Some(4),
        }
    }

    fn sample_year_scope(read_time: i64) -> StatisticsYearScope {
        StatisticsYearScope {
            summary: YearlySummary {
                completed_count: read_time,
            },
            daily_activity: vec![crate::models::DailyStats {
                date: "2026-03-03".to_string(),
                read_time,
                pages_read: read_time,
            }],
            monthly_aggregates: vec![MonthlyAggregate {
                month_key: "2026-03".to_string(),
                read_time,
                pages_read: read_time,
                active_days: 1,
            }],
            config: Some(StatisticsHeatmapConfig {
                max_scale_seconds: Some(read_time),
            }),
        }
    }

    #[test]
    fn statistics_index_scoped_projects_selected_scope() {
        let response = StatisticsIndexResponse {
            meta: sample_meta(),
            available_years: vec![2026],
            available_weeks: vec![AvailableWeek {
                week_key: "2026-03-02".to_string(),
                start_date: "2026-03-02".to_string(),
                end_date: "2026-03-08".to_string(),
            }],
            scopes: Scoped {
                all: StatisticsIndexScope {
                    overview: StatisticsOverview {
                        total_read_time: 1,
                        total_page_reads: 1,
                        longest_read_time_in_day: 1,
                        most_pages_in_day: 1,
                        average_session_duration: Some(1),
                        longest_session_duration: Some(1),
                        total_completions: 1,
                        books_completed: 1,
                        most_completions: 1,
                    },
                    streaks: StatisticsStreaks {
                        longest: crate::models::StreakInfo::new(
                            1,
                            Some("2026-03-01".to_string()),
                            Some("2026-03-01".to_string()),
                        ),
                        current: crate::models::StreakInfo::new(
                            1,
                            Some("2026-03-01".to_string()),
                            Some("2026-03-01".to_string()),
                        ),
                    },
                    heatmap_config: StatisticsHeatmapConfig {
                        max_scale_seconds: Some(1),
                    },
                },
                books: StatisticsIndexScope {
                    overview: StatisticsOverview {
                        total_read_time: 2,
                        total_page_reads: 2,
                        longest_read_time_in_day: 2,
                        most_pages_in_day: 2,
                        average_session_duration: Some(2),
                        longest_session_duration: Some(2),
                        total_completions: 2,
                        books_completed: 2,
                        most_completions: 2,
                    },
                    streaks: StatisticsStreaks {
                        longest: crate::models::StreakInfo::new(
                            2,
                            Some("2026-03-02".to_string()),
                            Some("2026-03-02".to_string()),
                        ),
                        current: crate::models::StreakInfo::new(
                            2,
                            Some("2026-03-02".to_string()),
                            Some("2026-03-02".to_string()),
                        ),
                    },
                    heatmap_config: StatisticsHeatmapConfig {
                        max_scale_seconds: Some(2),
                    },
                },
                comics: StatisticsIndexScope {
                    overview: StatisticsOverview {
                        total_read_time: 3,
                        total_page_reads: 3,
                        longest_read_time_in_day: 3,
                        most_pages_in_day: 3,
                        average_session_duration: Some(3),
                        longest_session_duration: Some(3),
                        total_completions: 3,
                        books_completed: 3,
                        most_completions: 3,
                    },
                    streaks: StatisticsStreaks {
                        longest: crate::models::StreakInfo::new(
                            3,
                            Some("2026-03-03".to_string()),
                            Some("2026-03-03".to_string()),
                        ),
                        current: crate::models::StreakInfo::new(
                            3,
                            Some("2026-03-03".to_string()),
                            Some("2026-03-03".to_string()),
                        ),
                    },
                    heatmap_config: StatisticsHeatmapConfig {
                        max_scale_seconds: Some(3),
                    },
                },
            },
        };

        let scoped = response.scoped(Scope::Books);
        assert_eq!(scoped.overview.total_read_time, 2);
        assert_eq!(scoped.streaks.longest.days, 2);
        assert_eq!(scoped.heatmap_config.max_scale_seconds, Some(2));

        let json = serde_json::to_value(scoped).expect("scoped stats index should serialize");
        assert!(json.get("scopes").is_none());
        assert!(json.get("overview").is_some());
    }

    #[test]
    fn statistics_week_scoped_flattens_selected_scope() {
        let response = StatisticsWeekResponse {
            meta: sample_meta(),
            week_key: "2026-03-02".to_string(),
            scopes: Scoped {
                all: sample_week(1),
                books: sample_week(2),
                comics: sample_week(3),
            },
        };

        let scoped = response.scoped(Scope::Comics);
        assert_eq!(scoped.stats.read_time, 3);

        let json = serde_json::to_value(scoped).expect("scoped week should serialize");
        assert!(json.get("scopes").is_none());
        assert_eq!(json.get("read_time").and_then(|v| v.as_i64()), Some(3));
    }

    #[test]
    fn statistics_year_scoped_flattens_selected_scope() {
        let response = StatisticsYearResponse {
            meta: sample_meta(),
            year: 2026,
            scopes: Scoped {
                all: sample_year_scope(1),
                books: sample_year_scope(2),
                comics: sample_year_scope(3),
            },
        };

        let scoped = response.scoped(Scope::All);
        assert_eq!(scoped.payload.summary.completed_count, 1);

        let json = serde_json::to_value(scoped).expect("scoped year should serialize");
        assert!(json.get("scopes").is_none());
        assert!(json.get("summary").is_some());
    }

    #[test]
    fn statistics_scoped_projection_matches_legacy_scope_projection() {
        let response = StatisticsWeekResponse {
            meta: sample_meta(),
            week_key: "2026-03-02".to_string(),
            scopes: Scoped {
                all: sample_week(11),
                books: sample_week(22),
                comics: sample_week(33),
            },
        };

        let legacy = project_scope_payload(
            serde_json::to_value(&response).expect("legacy payload should serialize"),
            Scope::Books,
        );
        let typed = serde_json::to_value(response.scoped(Scope::Books))
            .expect("typed scoped payload should serialize");

        assert_eq!(typed, legacy);
    }
}
