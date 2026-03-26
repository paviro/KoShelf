import type { PageActivityAnnotation, PageActivityEvent } from '../../../shared/contracts';

export interface AggregatedPage {
    page: number;
    totalDuration: number;
    readCount: number;
}

/**
 * Aggregate raw page events into per-page totals.
 * Optionally filter by a time range (unix timestamps, inclusive).
 */
export function aggregatePageData(
    events: PageActivityEvent[],
    startTime?: number,
    endTime?: number,
): Map<number, AggregatedPage> {
    const map = new Map<number, AggregatedPage>();
    for (const ev of events) {
        if (startTime !== undefined && ev.start_time < startTime) continue;
        if (endTime !== undefined && ev.start_time > endTime) continue;
        const existing = map.get(ev.page);
        if (existing) {
            existing.totalDuration += ev.duration;
            existing.readCount += 1;
        } else {
            map.set(ev.page, {
                page: ev.page,
                totalDuration: ev.duration,
                readCount: 1,
            });
        }
    }
    return map;
}

/**
 * Build annotation lookup: page number → list of annotations on that page.
 */
export function buildAnnotationMap(
    annotations: PageActivityAnnotation[],
): Map<number, PageActivityAnnotation[]> {
    const map = new Map<number, PageActivityAnnotation[]>();
    for (const a of annotations) {
        const list = map.get(a.page);
        if (list) {
            list.push(a);
        } else {
            map.set(a.page, [a]);
        }
    }
    return map;
}

/**
 * Compute a percentile value from page durations.
 * Reduces the impact of outlier pages on the color scale.
 */
export function percentileDuration(
    pageData: Map<number, AggregatedPage>,
    percentile: number,
): number {
    const durations = [...pageData.values()]
        .map((p) => p.totalDuration)
        .filter((d) => d > 0)
        .sort((a, b) => a - b);
    if (durations.length === 0) return 0;
    const index = Math.ceil((percentile / 100) * durations.length) - 1;
    return durations[Math.max(0, index)];
}

/**
 * Convert an ISO date string (YYYY-MM-DD) to a unix timestamp at the start of that day (UTC).
 */
export function isoDateToUnix(dateStr: string): number {
    return Math.floor(new Date(dateStr + 'T00:00:00Z').getTime() / 1000);
}

/**
 * Convert an ISO date string (YYYY-MM-DD) to a unix timestamp at the end of that day (UTC).
 */
export function isoDateToEndOfDayUnix(dateStr: string): number {
    return Math.floor(new Date(dateStr + 'T23:59:59Z').getTime() / 1000);
}
