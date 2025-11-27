use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

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
            let utc_dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
            let dt_tz = tz.from_utc_datetime(&utc_dt.naive_utc());
            let adjusted = dt_tz - Duration::minutes(self.day_start_minutes as i64);
            adjusted.date_naive()
        } else {
            let dt_local = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
                .with_timezone(&Local);
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

    /// Format current time as YYYY-MM-DD HH:MM in configured timezone (no day-start adjustment).
    pub fn now_formatted(&self) -> String {
        match self.timezone {
            Some(tz) => Utc::now()
                .with_timezone(&tz)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            None => Local::now().format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
