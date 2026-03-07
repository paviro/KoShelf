import { translation } from '../../../shared/i18n';
import {
    formatMonthKey,
    formatPlainDate,
    formatPlainDateRange,
} from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { RecapItemResponse } from '../api/recap-data';

export function formatRecapDuration(totalSeconds: number): string {
    if (!Number.isFinite(totalSeconds) || totalSeconds <= 0) {
        return `0${translation.get('units.m')}`;
    }

    const normalized = Math.floor(totalSeconds);
    const totalMinutes = Math.floor(normalized / 60);
    const days = Math.floor(totalMinutes / (24 * 60));
    const hours = Math.floor((totalMinutes % (24 * 60)) / 60);
    const minutes = totalMinutes % 60;

    const parts: string[] = [];
    if (days > 0) {
        parts.push(`${formatNumber(days)}${translation.get('units.d')}`);
    }
    if (hours > 0) {
        parts.push(`${formatNumber(hours)}${translation.get('units.h')}`);
    }
    if (minutes > 0 || parts.length === 0) {
        parts.push(`${formatNumber(minutes)}${translation.get('units.m')}`);
    }

    return parts.join(' ');
}

export function formatRecapHoursAndMinutes(
    hours: number,
    minutes: number,
): string {
    const normalizedHours = Number.isFinite(hours)
        ? Math.max(0, Math.floor(hours))
        : 0;
    const normalizedMinutes = Number.isFinite(minutes)
        ? Math.max(0, Math.floor(minutes))
        : 0;

    if (normalizedHours > 0 && normalizedMinutes > 0) {
        return `${formatNumber(normalizedHours)}${translation.get('units.h')} ${formatNumber(normalizedMinutes)}${translation.get('units.m')}`;
    }
    if (normalizedHours > 0) {
        return `${formatNumber(normalizedHours)}${translation.get('units.h')}`;
    }
    return `${formatNumber(normalizedMinutes)}${translation.get('units.m')}`;
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
    item: RecapItemResponse,
): '/books' | '/comics' {
    if (item.content_type === 'comic') {
        return '/comics';
    }

    return '/books';
}
