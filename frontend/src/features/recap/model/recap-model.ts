import { StorageManager } from '../../../shared/storage-manager';
import type { RecapMonthResponse, RecapScope } from '../api/recap-data';

const RECAP_YEAR_PARAM_REGEX = /^\d{4}$/;

export function normalizeRecapScope(scopeParam: string | undefined): RecapScope {
    if (scopeParam === 'books' || scopeParam === 'comics') {
        return scopeParam;
    }

    return 'all';
}

export function isRecapScopeParamCanonical(scopeParam: string | undefined): boolean {
    return scopeParam === undefined || scopeParam === 'books' || scopeParam === 'comics';
}

export function parseRecapYearParam(yearParam: string | undefined): number | null {
    if (!yearParam || !RECAP_YEAR_PARAM_REGEX.test(yearParam)) {
        return null;
    }

    const parsed = Number.parseInt(yearParam, 10);
    if (!Number.isFinite(parsed)) {
        return null;
    }

    return parsed;
}

export function resolveLatestYear(
    availableYears: number[],
    latestYear: number | null | undefined,
): number | null {
    if (typeof latestYear === 'number' && Number.isFinite(latestYear)) {
        return latestYear;
    }

    return availableYears[0] ?? null;
}

export function buildRecapPath(year: number | null, scope: RecapScope): string {
    if (year === null) {
        return '/recap';
    }

    if (scope === 'all') {
        return `/recap/${year}`;
    }

    return `/recap/${year}/${scope}`;
}

export function readStoredRecapScope(): RecapScope {
    const raw = StorageManager.get<string>(StorageManager.KEYS.RECAP_FILTER, 'all');
    if (raw === 'books' || raw === 'comics') {
        return raw;
    }

    return 'all';
}

export function persistRecapScope(scope: RecapScope): void {
    StorageManager.set(StorageManager.KEYS.RECAP_FILTER, scope);
}

export function readRecapSortNewest(): boolean {
    return StorageManager.get<boolean>(StorageManager.KEYS.RECAP_SORT_NEWEST, true) ?? true;
}

export function persistRecapSortNewest(value: boolean): void {
    StorageManager.set(StorageManager.KEYS.RECAP_SORT_NEWEST, value);
}

export function orderRecapMonths(
    months: RecapMonthResponse[],
    newestFirst: boolean,
): RecapMonthResponse[] {
    const copied = months.map((month) => ({
        ...month,
        items: [...month.items],
    }));

    if (newestFirst) {
        return copied.reverse();
    }

    return copied.map((month) => ({
        ...month,
        items: [...month.items].reverse(),
    }));
}
