export const STATISTICS_MONTH_KEYS = [
    'january',
    'february',
    'march',
    'april',
    'may',
    'june',
    'july',
    'august',
    'september',
    'october',
    'november',
    'december',
] as const;

export type StatisticsMonthKey = (typeof STATISTICS_MONTH_KEYS)[number];

export function monthKeyAt(index: number): StatisticsMonthKey {
    const safeIndex = Math.min(
        Math.max(index, 0),
        STATISTICS_MONTH_KEYS.length - 1,
    );
    return STATISTICS_MONTH_KEYS[safeIndex];
}

export function toShortMonthKey(
    monthKey: StatisticsMonthKey,
): `${StatisticsMonthKey}.short` {
    return `${monthKey}.short`;
}
