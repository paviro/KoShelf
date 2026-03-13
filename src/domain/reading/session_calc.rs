use std::collections::HashMap;

use crate::models::PageStat;

/// Default time gap that separates two reading events into different sessions (in seconds)
const DEFAULT_SESSION_GAP_SECONDS: i64 = 300; // 5 minutes

/// Calculate the duration (in seconds) of each reading session for a single book.
/// Two consecutive page reads belong to the same session when the gap between
/// them is less than or equal to the default gap (5 minutes).
pub fn session_durations(stats: &[PageStat]) -> Vec<i64> {
    if stats.is_empty() {
        return Vec::new();
    }

    let mut sorted = stats.to_vec();
    sorted.sort_by_key(|s| s.start_time);

    let mut durations = Vec::new();
    let mut current = sorted[0].duration;
    let mut last_end = sorted[0].start_time + sorted[0].duration;

    for stat in &sorted[1..] {
        if stat.start_time - last_end <= DEFAULT_SESSION_GAP_SECONDS {
            current += stat.duration;
        } else {
            durations.push(current);
            current = stat.duration;
        }
        last_end = stat.start_time + stat.duration;
    }

    durations.push(current);
    durations
}

/// Convenience helper that only returns the number of sessions.
#[inline]
pub fn session_count(stats: &[PageStat]) -> i64 {
    session_durations(stats).len() as i64
}

/// Aggregate session durations across *all* books contained in `page_stats`.
/// The returned vector contains the duration (in seconds) of **every** session
/// across the dataset.
pub fn aggregate_session_durations(page_stats: &[PageStat]) -> Vec<i64> {
    let mut by_book: HashMap<i64, Vec<PageStat>> = HashMap::new();
    for stat in page_stats.iter().filter(|s| s.duration > 0) {
        by_book.entry(stat.id_book).or_default().push(stat.clone());
    }

    let mut all = Vec::new();
    for stats in by_book.values() {
        let mut durations = session_durations(stats);
        all.append(&mut durations);
    }

    all
}

/// Compute (average_session_duration, longest_session_duration) from the provided
/// `page_stats` slice. Returns `(None, None)` if no valid sessions exist.
pub fn session_metrics(page_stats: &[PageStat]) -> (Option<i64>, Option<i64>) {
    let sessions = aggregate_session_durations(page_stats);
    if sessions.is_empty() {
        return (None, None);
    }
    let total: i64 = sessions.iter().sum();
    let average = Some(total / sessions.len() as i64);
    let longest = sessions.iter().max().copied();
    (average, longest)
}
