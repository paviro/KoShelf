import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { LibrarySeries } from '../api/library-data';

const FALLBACK_LOCALE = 'en-US';
const ISO_DATE_REGEX = /^(\d{4})-(\d{2})-(\d{2})$/;
const ISO_DATETIME_REGEX =
    /^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})(?::(\d{2}))?$/;

function isFiniteNumber(value: unknown): value is number {
    return typeof value === 'number' && Number.isFinite(value);
}

function currentLocale(): string {
    return translation.getLanguage() || FALLBACK_LOCALE;
}

function safeDateFormat(
    date: Date,
    options: Intl.DateTimeFormatOptions,
): string {
    try {
        return new Intl.DateTimeFormat(currentLocale(), options).format(date);
    } catch {
        return new Intl.DateTimeFormat(FALLBACK_LOCALE, options).format(date);
    }
}

function parseIsoDate(value: string): Date | null {
    const match = ISO_DATE_REGEX.exec(value.trim());
    if (!match) {
        return null;
    }

    const year = Number.parseInt(match[1], 10);
    const month = Number.parseInt(match[2], 10);
    const day = Number.parseInt(match[3], 10);

    if (
        !Number.isFinite(year) ||
        !Number.isFinite(month) ||
        !Number.isFinite(day)
    ) {
        return null;
    }

    return new Date(Date.UTC(year, month - 1, day, 12, 0, 0, 0));
}

function parseIsoDateTime(value: string): Date | null {
    const match = ISO_DATETIME_REGEX.exec(value.trim());
    if (!match) {
        return null;
    }

    const year = Number.parseInt(match[1], 10);
    const month = Number.parseInt(match[2], 10);
    const day = Number.parseInt(match[3], 10);
    const hour = Number.parseInt(match[4], 10);
    const minute = Number.parseInt(match[5], 10);
    const second = Number.parseInt(match[6] ?? '0', 10);

    if (
        !Number.isFinite(year) ||
        !Number.isFinite(month) ||
        !Number.isFinite(day) ||
        !Number.isFinite(hour) ||
        !Number.isFinite(minute) ||
        !Number.isFinite(second)
    ) {
        return null;
    }

    return new Date(Date.UTC(year, month - 1, day, hour, minute, second));
}

export function toProgressPercentage(
    progress: number | null | undefined,
): number {
    if (!isFiniteNumber(progress)) {
        return 0;
    }

    const percent = progress * 100;
    return Math.min(100, Math.max(0, Math.round(percent)));
}

export function formatSeriesDisplay(
    series: LibrarySeries | null | undefined,
): string {
    const normalizedName = (series?.name ?? '').trim();
    if (!normalizedName) {
        return '';
    }

    const normalizedIndex = (series?.index ?? '').trim();
    if (!normalizedIndex) {
        return normalizedName;
    }

    return `${normalizedName} #${normalizedIndex}`;
}

export function formatDurationFromSeconds(
    seconds: number | null | undefined,
): string {
    if (!isFiniteNumber(seconds)) {
        return '--';
    }

    const normalizedSeconds = Math.max(0, Math.floor(seconds));
    const totalMinutes = Math.floor(normalizedSeconds / 60);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    if (hours > 0) {
        return `${formatNumber(hours)}${translation.get('units.h')} ${formatNumber(minutes)}${translation.get('units.m')}`;
    }

    return `${formatNumber(totalMinutes)}${translation.get('units.m')}`;
}

export function formatCompletionDateRange(
    startDate: string,
    endDate: string,
): string {
    const start = parseIsoDate(startDate);
    const end = parseIsoDate(endDate);

    if (!start || !end) {
        return startDate === endDate ? startDate : `${startDate} – ${endDate}`;
    }

    const currentYear = new Date().getUTCFullYear();
    const formatOne = (date: Date): string => {
        const includeYear = date.getUTCFullYear() !== currentYear;
        return safeDateFormat(date, {
            month: 'short',
            day: 'numeric',
            ...(includeYear ? { year: 'numeric' as const } : {}),
            timeZone: 'UTC',
        });
    };

    if (startDate === endDate) {
        return formatOne(start);
    }

    return `${formatOne(start)} – ${formatOne(end)}`;
}

export function formatIsoDate(value: string | null | undefined): string {
    if (!value) {
        return '--';
    }

    const parsed = parseIsoDate(value);
    if (!parsed) {
        return value;
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

export function formatAnnotationDatetime(
    value: string | null | undefined,
): string | null {
    if (!value) {
        return null;
    }

    const parsed = parseIsoDateTime(value);
    if (!parsed) {
        return value;
    }

    return safeDateFormat(parsed, {
        dateStyle: 'long',
        timeStyle: 'short',
        timeZone: 'UTC',
    });
}

export function formatLanguageDisplayName(
    value: string | null | undefined,
): string {
    if (!value || !value.trim()) {
        return '--';
    }

    const normalized = value.trim().replaceAll('_', '-');
    const [baseLanguage] = normalized.split('-');

    if (!baseLanguage) {
        return normalized.toUpperCase();
    }

    try {
        const displayNames = new Intl.DisplayNames([currentLocale()], {
            type: 'language',
        });
        return (
            displayNames.of(baseLanguage.toLowerCase()) ??
            normalized.toUpperCase()
        );
    } catch {
        return normalized.toUpperCase();
    }
}

export function formatReadingSpeed(value: number | null | undefined): string {
    if (!isFiniteNumber(value)) {
        return '--';
    }

    return formatNumber(Math.max(0, value), {
        minimumFractionDigits: 1,
        maximumFractionDigits: 1,
    });
}

export function calculateAverageReadingSpeed(
    pagesRead: number,
    readingTimeSeconds: number,
): number | null {
    if (!Number.isFinite(pagesRead) || !Number.isFinite(readingTimeSeconds)) {
        return null;
    }

    if (pagesRead <= 0 || readingTimeSeconds <= 0) {
        return null;
    }

    return pagesRead / (readingTimeSeconds / 3600);
}

export function calculateCalendarLengthDays(
    startDate: string,
    endDate: string,
): number | null {
    const start = parseIsoDate(startDate);
    const end = parseIsoDate(endDate);

    if (!start || !end) {
        return null;
    }

    const millisecondsPerDay = 24 * 60 * 60 * 1000;
    const delta = Math.round(
        Math.abs(end.getTime() - start.getTime()) / millisecondsPerDay,
    );
    return delta + 1;
}

export function sanitizeRichTextHtml(rawHtml: string): string {
    if (!rawHtml.trim()) {
        return '';
    }

    if (typeof window === 'undefined' || typeof DOMParser === 'undefined') {
        return rawHtml;
    }

    const parsed = new DOMParser().parseFromString(rawHtml, 'text/html');
    parsed
        .querySelectorAll('script, style, iframe, object, embed')
        .forEach((node) => {
            node.remove();
        });

    parsed.querySelectorAll('*').forEach((element) => {
        for (const attribute of Array.from(element.attributes)) {
            const attributeName = attribute.name.toLowerCase();
            const attributeValue = attribute.value.trim().toLowerCase();

            if (attributeName.startsWith('on')) {
                element.removeAttribute(attribute.name);
                continue;
            }

            if (
                (attributeName === 'href' || attributeName === 'src') &&
                (attributeValue.startsWith('javascript:') ||
                    attributeValue.startsWith('data:') ||
                    attributeValue.startsWith('vbscript:'))
            ) {
                element.removeAttribute(attribute.name);
            }
        }
    });

    return parsed.body.innerHTML;
}
