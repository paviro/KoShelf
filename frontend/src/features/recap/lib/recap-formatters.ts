import {
    formatMonthKey,
    formatPlainDateRange,
} from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { CompletionItem } from '../api/recap-data';

export function formatRecapDateRange(
    startDateIso: string,
    endDateIso: string,
): string {
    return formatPlainDateRange(startDateIso, endDateIso, {
        monthStyle: 'short',
        yearDisplay: 'auto',
    });
}

export function formatRecapMonth(monthKey: string): string {
    return formatMonthKey(monthKey, {
        monthStyle: 'long',
        includeYear: false,
    });
}

export function formatRecapPercentage(value: number): string {
    if (!Number.isFinite(value)) {
        return '0';
    }

    const rounded = Math.round(value * 10) / 10;
    if (Number.isInteger(rounded)) {
        return formatNumber(rounded);
    }

    return formatNumber(rounded, {
        minimumFractionDigits: 1,
        maximumFractionDigits: 1,
    });
}

export function buildStarDisplay(rating: number | null | undefined): boolean[] {
    const normalized = Number.isFinite(rating)
        ? Math.max(0, Math.floor(rating ?? 0))
        : 0;
    return [1, 2, 3, 4, 5].map((threshold) => normalized >= threshold);
}

export function resolveRecapSearchBasePath(
    item: CompletionItem,
): '/books' | '/comics' {
    if (item.content_type === 'comic') {
        return '/comics';
    }

    return '/books';
}
