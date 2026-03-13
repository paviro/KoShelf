import { translation } from '../../../shared/i18n';
import {
    formatMonthKey,
    formatPlainDate,
    formatPlainDateRange,
} from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import {
    joinUnitValueParts,
    type UnitValuePart,
} from '../../../shared/lib/intl/unit-value';
import type { CompletionItem } from '../api/recap-data';

export function formatRecapDuration(totalSeconds: number): string {
    return joinUnitValueParts(formatRecapDurationParts(totalSeconds));
}

export function formatRecapDurationParts(
    totalSeconds: number,
): UnitValuePart[] {
    if (!Number.isFinite(totalSeconds) || totalSeconds <= 0) {
        return [{ amount: '0', unit: translation.get('units.m') }];
    }

    const normalized = Math.floor(totalSeconds);
    const totalMinutes = Math.floor(normalized / 60);
    const days = Math.floor(totalMinutes / (24 * 60));
    const hours = Math.floor((totalMinutes % (24 * 60)) / 60);
    const minutes = totalMinutes % 60;

    const parts: UnitValuePart[] = [];
    if (days > 0) {
        parts.push({
            amount: formatNumber(days),
            unit: translation.get('units.d'),
        });
    }
    if (hours > 0) {
        parts.push({
            amount: formatNumber(hours),
            unit: translation.get('units.h'),
        });
    }
    if (minutes > 0 || parts.length === 0) {
        parts.push({
            amount: formatNumber(minutes),
            unit: translation.get('units.m'),
        });
    }

    return parts;
}

export function formatRecapDate(dateIso: string): string {
    return formatPlainDate(dateIso, {
        monthStyle: 'short',
        yearDisplay: 'auto',
    });
}

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
