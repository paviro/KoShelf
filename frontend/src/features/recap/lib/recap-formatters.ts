import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { RecapItemResponse } from '../api/recap-data';

const FALLBACK_LOCALE = 'en-US';
const ISO_DATE_REGEX = /^(\d{4})-(\d{2})-(\d{2})$/;

function parseIsoDate(value: string): Date | null {
    const match = ISO_DATE_REGEX.exec(value.trim());
    if (!match) {
        return null;
    }

    const year = Number.parseInt(match[1], 10);
    const month = Number.parseInt(match[2], 10);
    const day = Number.parseInt(match[3], 10);
    if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
        return null;
    }

    return new Date(Date.UTC(year, month - 1, day, 12, 0, 0, 0));
}

function safeDateFormat(date: Date, options: Intl.DateTimeFormatOptions): string {
    const locale = translation.getLanguage() || FALLBACK_LOCALE;
    try {
        return new Intl.DateTimeFormat(locale, options).format(date);
    } catch {
        return new Intl.DateTimeFormat(FALLBACK_LOCALE, options).format(date);
    }
}

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

export function formatRecapHoursAndMinutes(hours: number, minutes: number): string {
    const normalizedHours = Number.isFinite(hours) ? Math.max(0, Math.floor(hours)) : 0;
    const normalizedMinutes = Number.isFinite(minutes) ? Math.max(0, Math.floor(minutes)) : 0;

    if (normalizedHours > 0 && normalizedMinutes > 0) {
        return `${formatNumber(normalizedHours)}${translation.get('units.h')} ${formatNumber(normalizedMinutes)}${translation.get('units.m')}`;
    }
    if (normalizedHours > 0) {
        return `${formatNumber(normalizedHours)}${translation.get('units.h')}`;
    }
    return `${formatNumber(normalizedMinutes)}${translation.get('units.m')}`;
}

export function formatRecapDate(dateIso: string): string {
    const parsed = parseIsoDate(dateIso);
    if (!parsed) {
        return dateIso;
    }

    const currentYear = new Date().getUTCFullYear();
    const includeYear = parsed.getUTCFullYear() !== currentYear;

    return safeDateFormat(parsed, {
        month: 'short',
        day: 'numeric',
        ...(includeYear ? { year: 'numeric' as const } : {}),
        timeZone: 'UTC',
    });
}

export function formatRecapDateRange(startDateIso: string, endDateIso: string): string {
    if (startDateIso === endDateIso) {
        return formatRecapDate(startDateIso);
    }

    return `${formatRecapDate(startDateIso)} – ${formatRecapDate(endDateIso)}`;
}

export function formatRecapMonthLabel(monthLabel: string): string {
    const key = monthLabel.trim().toLowerCase();
    const translated = translation.get(key);
    return translated === key ? monthLabel : translated;
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
    const normalized = Number.isFinite(rating) ? Math.max(0, Math.floor(rating ?? 0)) : 0;
    return [1, 2, 3, 4, 5].map((threshold) => normalized >= threshold);
}

export function resolveRecapSearchBasePath(item: RecapItemResponse): '/books' | '/comics' {
    if (item.content_type === 'comic') {
        return '/comics';
    }

    return '/books';
}
