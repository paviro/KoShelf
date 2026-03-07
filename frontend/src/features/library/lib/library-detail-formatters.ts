import { translation } from '../../../shared/i18n';
import {
    formatInstant,
    formatPlainDate,
    formatPlainDateRange,
    parsePlainDate,
} from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { LibrarySeries } from '../api/library-data';

function isFiniteNumber(value: unknown): value is number {
    return typeof value === 'number' && Number.isFinite(value);
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
    return formatPlainDateRange(startDate, endDate, {
        monthStyle: 'short',
        yearDisplay: 'auto',
    });
}

export function formatIsoDate(value: string | null | undefined): string {
    return formatPlainDate(value, {
        monthStyle: 'short',
        yearDisplay: 'auto',
    });
}

export function formatAnnotationDatetime(
    value: string | null | undefined,
): string | null {
    if (!value) {
        return null;
    }

    return formatInstant(value, {
        dateStyle: 'long',
        timeStyle: 'short',
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
        const displayNames = new Intl.DisplayNames(
            [translation.getLanguage() || 'en-US'],
            {
                type: 'language',
            },
        );
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
    const start = parsePlainDate(startDate);
    const end = parsePlainDate(endDate);

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
