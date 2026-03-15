use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

/// Timezone and logical day-start offset used to map Unix timestamps to reading dates.
///
/// When `timezone` is `None`, the system local timezone is used.
/// `day_start_minutes` shifts the day boundary (e.g. 240 = 04:00, so reading at
/// 02:00 counts as the previous calendar day).
#[derive(Clone, Debug)]
pub struct TimeConfig {
    pub timezone: Option<Tz>,
    pub day_start_minutes: u16,
}

impl TimeConfig {
    pub fn new(timezone: Option<Tz>, day_start_minutes: u16) -> Self {
        Self {
            timezone,
            day_start_minutes,
        }
    }

    /// Build from optional CLI strings (timezone IANA name, day start as HH:MM)
    pub fn from_cli(timezone: &Option<String>, day_start_time: &Option<String>) -> Result<Self> {
        let tz = match timezone {
            Some(tz_str) if !tz_str.trim().is_empty() => {
                let parsed: Tz = tz_str.parse().map_err(|_| {
                    anyhow!("Invalid timezone: {}. Example: Australia/Sydney", tz_str)
                })?;
                Some(parsed)
            }
            _ => None,
        };

        let minutes = match day_start_time {
            Some(s) if !s.trim().is_empty() => Self::parse_day_start_minutes(s)?,
            _ => 0u16,
        };

        Ok(Self::new(tz, minutes))
    }

    fn parse_day_start_minutes(s: &str) -> Result<u16> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid --day-start-time format. Use HH:MM (e.g., 03:00)"
            ));
        }
        let hours: u16 = parts[0]
            .parse()
            .map_err(|_| anyhow!("Invalid hour in --day-start-time"))?;
        let minutes: u16 = parts[1]
            .parse()
            .map_err(|_| anyhow!("Invalid minute in --day-start-time"))?;
        if hours > 23 || minutes > 59 {
            return Err(anyhow!("--day-start-time must be between 00:00 and 23:59"));
        }
        Ok(hours * 60 + minutes)
    }

    /// Get the logical local date for a Unix timestamp, applying timezone and day-start offset.
    pub fn date_for_timestamp(&self, timestamp: i64) -> NaiveDate {
        if let Some(tz) = self.timezone {
            let utc_dt = Self::timestamp_utc(timestamp);
            let dt_tz = tz.from_utc_datetime(&utc_dt.naive_utc());
            let adjusted = dt_tz - Duration::minutes(self.day_start_minutes as i64);
            adjusted.date_naive()
        } else {
            let dt_local = Self::timestamp_utc(timestamp).with_timezone(&Local);
            let adjusted = dt_local - Duration::minutes(self.day_start_minutes as i64);
            adjusted.date_naive()
        }
    }

    /// Format a timestamp as YYYY-MM-DD under configured timezone/day-start.
    pub fn format_date(&self, timestamp: i64) -> String {
        self.date_for_timestamp(timestamp)
            .format("%Y-%m-%d")
            .to_string()
    }

    /// Today's logical date under configured timezone/day-start.
    pub fn today_date(&self) -> NaiveDate {
        let now_ts = Utc::now().timestamp();
        self.date_for_timestamp(now_ts)
    }

    fn timestamp_utc(timestamp: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
    }

    fn parse_naive_datetime(value: &str) -> Option<NaiveDateTime> {
        let normalized = value.trim();
        for pattern in [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M",
        ] {
            if let Ok(parsed) = NaiveDateTime::parse_from_str(normalized, pattern) {
                return Some(parsed);
            }
        }

        None
    }

    fn choose_local_datetime<Tz>(result: LocalResult<DateTime<Tz>>) -> Option<DateTime<Tz>>
    where
        Tz: TimeZone,
    {
        match result {
            LocalResult::Single(date_time) => Some(date_time),
            LocalResult::Ambiguous(earliest, _) => Some(earliest),
            LocalResult::None => None,
        }
    }

    /// Format a Unix timestamp as RFC3339 in the configured timezone.
    pub fn format_timestamp_rfc3339(&self, timestamp: i64) -> String {
        let utc_dt = Self::timestamp_utc(timestamp);
        match self.timezone {
            Some(tz) => tz
                .from_utc_datetime(&utc_dt.naive_utc())
                .format("%Y-%m-%dT%H:%M:%S%:z")
                .to_string(),
            None => utc_dt
                .with_timezone(&Local)
                .format("%Y-%m-%dT%H:%M:%S%:z")
                .to_string(),
        }
    }

    /// Format current time as an RFC3339 instant in the configured timezone.
    pub fn now_rfc3339(&self) -> String {
        self.format_timestamp_rfc3339(Utc::now().timestamp())
    }

    /// Interpret a timezone-less wall-clock datetime under the configured timezone.
    pub fn normalize_naive_datetime_to_rfc3339(&self, value: &str) -> Option<String> {
        let parsed = Self::parse_naive_datetime(value)?;
        match self.timezone {
            Some(tz) => Self::choose_local_datetime(tz.from_local_datetime(&parsed))
                .map(|date_time| date_time.format("%Y-%m-%dT%H:%M:%S%:z").to_string()),
            None => Self::choose_local_datetime(Local.from_local_datetime(&parsed))
                .map(|date_time| date_time.format("%Y-%m-%dT%H:%M:%S%:z").to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimeConfig;

    #[test]
    fn format_timestamp_rfc3339_respects_configured_timezone() {
        let timezone = "Europe/Berlin".parse().expect("timezone should parse");
        let config = TimeConfig::new(Some(timezone), 0);

        assert_eq!(
            config.format_timestamp_rfc3339(0),
            "1970-01-01T01:00:00+01:00"
        );
    }

    #[test]
    fn normalize_naive_datetime_uses_configured_timezone() {
        let timezone = "UTC".parse().expect("timezone should parse");
        let config = TimeConfig::new(Some(timezone), 0);

        assert_eq!(
            config.normalize_naive_datetime_to_rfc3339("2024-03-15 14:30:00"),
            Some("2024-03-15T14:30:00+00:00".to_string())
        );
    }
}
