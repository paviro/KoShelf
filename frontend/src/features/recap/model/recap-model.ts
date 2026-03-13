import {
    patchRouteState,
    readRouteState,
} from '../../../shared/lib/state/route-state-storage';
import type { CompletionGroup, RecapScope } from '../api/recap-data';

export type RecapViewState = {
    scope: RecapScope;
    year: number | null;
};

function normalizeRecapYear(value: unknown): number | null {
    if (typeof value !== 'number' || !Number.isFinite(value)) {
        return null;
    }

    const rounded = Math.floor(value);
    if (rounded < 1900 || rounded > 9999) {
        return null;
    }

    return rounded;
}

export function normalizeRecapScope(scopeParam: unknown): RecapScope {
    if (scopeParam === 'books' || scopeParam === 'comics') {
        return scopeParam;
    }

    return 'all';
}

export function readStoredRecapViewState(): RecapViewState {
    const persisted = readRouteState('recap', 'session');
    return {
        scope: normalizeRecapScope(persisted.scope),
        year: normalizeRecapYear(persisted.year),
    };
}

export function persistRecapViewState(state: RecapViewState): void {
    patchRouteState('recap', 'session', {
        scope: normalizeRecapScope(state.scope),
        year: normalizeRecapYear(state.year),
    });
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

export function readStoredRecapScope(): RecapScope {
    return readStoredRecapViewState().scope;
}

export function readStoredRecapYear(): number | null {
    return readStoredRecapViewState().year;
}

export function readRecapSortNewest(): boolean {
    const persisted = readRouteState('recap', 'local');
    return typeof persisted.sortNewestFirst === 'boolean'
        ? persisted.sortNewestFirst
        : true;
}

export function persistRecapSortNewest(value: boolean): void {
    patchRouteState('recap', 'local', {
        sortNewestFirst: value,
    });
}

export function orderRecapMonths(
    months: CompletionGroup[],
    newestFirst: boolean,
): CompletionGroup[] {
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
