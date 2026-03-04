export const HEATMAP_COLOR_CLASSES = [
    ['bg-gray-100', 'dark:bg-dark-800'],
    ['bg-green-100', 'dark:bg-green-900'],
    ['bg-green-300', 'dark:bg-green-700'],
    ['bg-green-500', 'dark:bg-green-500'],
    ['bg-green-600', 'dark:bg-green-300'],
] as const;

export function calculateCellDate(
    year: number,
    weekIndex: number,
    dayIndex: number,
): Date {
    const janFirst = new Date(year, 0, 1);
    const janDayOfWeek = janFirst.getDay();
    const shiftToMonday = janDayOfWeek === 0 ? -6 : 1 - janDayOfWeek;
    const firstMonday = new Date(janFirst);
    firstMonday.setDate(janFirst.getDate() + shiftToMonday);

    const cellDate = new Date(firstMonday);
    cellDate.setDate(cellDate.getDate() + weekIndex * 7 + dayIndex);
    return cellDate;
}

export function formatISODate(date: Date): string {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
        date.getDate(),
    ).padStart(2, '0')}`;
}

export function normalizeHeatmapLevel(
    activity: number,
    maxActivity: number,
): number {
    const maxLevel = HEATMAP_COLOR_CLASSES.length - 1;
    if (!Number.isFinite(activity) || activity <= 0) {
        return 0;
    }
    if (!Number.isFinite(maxActivity) || maxActivity <= 0) {
        return maxLevel;
    }

    if (maxActivity <= maxLevel) {
        return Math.min(maxLevel, Math.max(1, Math.ceil(activity)));
    }

    const scaled = Math.ceil((activity / maxActivity) * maxLevel);
    return Math.min(maxLevel, Math.max(1, scaled));
}
