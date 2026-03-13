import { translation } from '../../i18n';
import { resolveLocalePatternContext } from './locale-options';

const FALLBACK_LOCALE = 'en-US';
const PLAIN_DATE_REGEX = /^(\d{4})-(\d{2})-(\d{2})$/;
const MONTH_KEY_REGEX = /^(\d{4})-(\d{2})$/;
const TEXT_BEARING_LITERAL_REGEX = /[\p{L}\p{M}]/u;
const DATE_PART_TYPES = new Set([
    'day',
    'weekday',
    'month',
    'year',
    'era',
    'relatedYear',
    'yearName',
]);
const TIME_PART_TYPES = new Set([
    'dayPeriod',
    'hour',
    'minute',
    'second',
    'fractionalSecond',
]);
type MonthStyle = 'short' | 'long';
type YearDisplay = 'auto' | 'always' | 'never';
type DateStyle = 'full' | 'long' | 'medium' | 'short';

type PlainDateFormatOptions = {
    monthStyle?: MonthStyle;
    yearDisplay?: YearDisplay;
    fallback?: string;
};

type MonthKeyFormatOptions = {
    monthStyle?: MonthStyle;
    includeYear?: boolean;
    fallback?: string;
};

type InstantFormatOptions = {
    dateStyle?: DateStyle;
    timeStyle?: DateStyle;
    fallback?: string;
};

function currentLocale(): string {
    return translation.getLanguage() || FALLBACK_LOCALE;
}

function safeFormatter(
    options: Intl.DateTimeFormatOptions,
    locale = currentLocale(),
): Intl.DateTimeFormat {
    try {
        return new Intl.DateTimeFormat(locale, options);
    } catch {
        return new Intl.DateTimeFormat(FALLBACK_LOCALE, options);
    }
}

function isTextBearingLiteral(value: string): boolean {
    return TEXT_BEARING_LITERAL_REGEX.test(value);
}

function hasTextBearingLiterals(parts: Intl.DateTimeFormatPart[]): boolean {
    return parts.some(
        (part) => part.type === 'literal' && isTextBearingLiteral(part.value),
    );
}

function compactDateParts(
    parts: Intl.DateTimeFormatPart[],
): Intl.DateTimeFormatPart[] {
    const compacted: Intl.DateTimeFormatPart[] = [];
    for (const part of parts) {
        if (part.type === 'literal') {
            if (!part.value) {
                continue;
            }

            const previous = compacted[compacted.length - 1];
            if (previous?.type === 'literal') {
                previous.value += part.value;
                continue;
            }
        }

        compacted.push({ ...part });
    }

    return compacted;
}

function getPartKind(type: string): 'date' | 'time' | 'other' {
    if (DATE_PART_TYPES.has(type)) {
        return 'date';
    }

    if (TIME_PART_TYPES.has(type)) {
        return 'time';
    }

    return 'other';
}

function findAdjacentPartType(
    parts: Intl.DateTimeFormatPart[],
    startIndex: number,
    step: -1 | 1,
): string | null {
    for (
        let index = startIndex + step;
        index >= 0 && index < parts.length;
        index += step
    ) {
        const part = parts[index];
        if (part.type !== 'literal') {
            return part.type;
        }
    }

    return null;
}

function neutralLiteralValue(
    patternParts: Intl.DateTimeFormatPart[],
    literalIndex: number,
): string {
    const previousType = findAdjacentPartType(patternParts, literalIndex, -1);
    const nextType = findAdjacentPartType(patternParts, literalIndex, 1);
    if (!previousType || !nextType) {
        return '';
    }

    const previousKind = getPartKind(previousType);
    const nextKind = getPartKind(nextType);
    if (
        (previousKind === 'date' && nextKind === 'time') ||
        (previousKind === 'time' && nextKind === 'date')
    ) {
        return ', ';
    }

    if (previousType === 'weekday' || nextType === 'weekday') {
        return ', ';
    }

    return ' ';
}

function normalizePatternLiterals(
    patternParts: Intl.DateTimeFormatPart[],
): Intl.DateTimeFormatPart[] {
    return compactDateParts(
        patternParts.map((part, index) => {
            if (part.type !== 'literal' || !isTextBearingLiteral(part.value)) {
                return part;
            }

            return {
                type: 'literal',
                value: neutralLiteralValue(patternParts, index),
            };
        }),
    );
}

function mergeDateParts(
    patternParts: Intl.DateTimeFormatPart[],
    valueParts: Intl.DateTimeFormatPart[],
): Intl.DateTimeFormatPart[] {
    const valuePartsByType = new Map<string, Intl.DateTimeFormatPart[]>();
    for (const part of valueParts) {
        if (part.type === 'literal') {
            continue;
        }

        const bucket = valuePartsByType.get(part.type) ?? [];
        bucket.push(part);
        valuePartsByType.set(part.type, bucket);
    }

    const usedCounts = new Map<string, number>();
    return compactDateParts(
        patternParts.map((part) => {
            if (part.type === 'literal') {
                return part;
            }

            const usedCount = usedCounts.get(part.type) ?? 0;
            usedCounts.set(part.type, usedCount + 1);

            const matchingPart = valuePartsByType.get(part.type)?.[usedCount];
            if (!matchingPart) {
                return part;
            }

            return {
                type: part.type,
                value: matchingPart.value,
            };
        }),
    );
}

function formatParts(parts: Intl.DateTimeFormatPart[]): string {
    return parts.map((part) => part.value).join('');
}

function formatDateObjectToMergedParts(
    value: Date,
    options: Intl.DateTimeFormatOptions,
    locale = currentLocale(),
): Intl.DateTimeFormatPart[] {
    const resolved = resolveLocalePatternContext(locale);
    if (!resolved.patternLocale) {
        return safeFormatter(options, locale).formatToParts(value);
    }

    const patternParts = safeFormatter(
        options,
        resolved.patternLocale,
    ).formatToParts(value);
    const valueParts = safeFormatter(
        options,
        resolved.valueLocale,
    ).formatToParts(value);

    if (!hasTextBearingLiterals(patternParts)) {
        return mergeDateParts(patternParts, valueParts);
    }

    // Preserve the region's field order without leaking foreign-language
    // connective words like "de", "à", or "alle ore".
    return mergeDateParts(normalizePatternLiterals(patternParts), valueParts);
}

function formatDateObjectToString(
    value: Date,
    options: Intl.DateTimeFormatOptions,
    locale = currentLocale(),
): string {
    return formatParts(formatDateObjectToMergedParts(value, options, locale));
}

function isValidDate(date: Date): boolean {
    return Number.isFinite(date.getTime());
}

function parseDateParts(
    value: string,
    pattern: RegExp,
): { year: number; month: number; day?: number } | null {
    const match = pattern.exec(value.trim());
    if (!match) {
        return null;
    }

    const year = Number.parseInt(match[1], 10);
    const month = Number.parseInt(match[2], 10);
    const day = match[3] ? Number.parseInt(match[3], 10) : undefined;

    if (
        !Number.isFinite(year) ||
        !Number.isFinite(month) ||
        month < 1 ||
        month > 12 ||
        (day !== undefined && (!Number.isFinite(day) || day < 1 || day > 31))
    ) {
        return null;
    }

    return { year, month, day };
}

function plainDateFormatOptions(
    startDate: Date,
    yearDisplay: YearDisplay,
    endDate?: Date,
    monthStyle: MonthStyle = 'short',
): Intl.DateTimeFormatOptions {
    const currentYear = new Date().getUTCFullYear();
    const includeYear =
        yearDisplay === 'always' ||
        (yearDisplay === 'auto' &&
            (startDate.getUTCFullYear() !== currentYear ||
                (endDate !== undefined &&
                    (endDate.getUTCFullYear() !== currentYear ||
                        endDate.getUTCFullYear() !==
                            startDate.getUTCFullYear()))));

    return {
        month: monthStyle,
        day: 'numeric',
        ...(includeYear ? { year: 'numeric' as const } : {}),
        timeZone: 'UTC',
    };
}

export function parsePlainDate(value: string): Date | null {
    const parts = parseDateParts(value, PLAIN_DATE_REGEX);
    if (!parts || parts.day === undefined) {
        return null;
    }

    return new Date(
        Date.UTC(parts.year, parts.month - 1, parts.day, 12, 0, 0, 0),
    );
}

export function parseMonthKey(value: string): Date | null {
    const parts = parseDateParts(value, MONTH_KEY_REGEX);
    if (!parts) {
        return null;
    }

    return new Date(Date.UTC(parts.year, parts.month - 1, 1, 12, 0, 0, 0));
}

export function parseInstant(value: string): Date | null {
    const parsed = new Date(value.trim());
    return isValidDate(parsed) ? parsed : null;
}

export function formatPlainDate(
    value: string | null | undefined,
    options: PlainDateFormatOptions = {},
): string {
    const fallback = options.fallback ?? '--';
    if (!value) {
        return fallback;
    }

    const parsed = parsePlainDate(value);
    if (!parsed) {
        return value;
    }

    return formatDateObjectToString(
        parsed,
        plainDateFormatOptions(
            parsed,
            options.yearDisplay ?? 'auto',
            undefined,
            options.monthStyle ?? 'short',
        ),
    );
}

export function formatPlainDateRange(
    startValue: string,
    endValue: string,
    options: PlainDateFormatOptions = {},
): string {
    if (startValue === endValue) {
        return formatPlainDate(startValue, options);
    }

    const start = parsePlainDate(startValue);
    const end = parsePlainDate(endValue);
    if (!start || !end) {
        return `${startValue} – ${endValue}`;
    }

    const locale = currentLocale();
    const formatOptions = plainDateFormatOptions(
        start,
        options.yearDisplay ?? 'auto',
        end,
        options.monthStyle ?? 'short',
    );
    if (!resolveLocalePatternContext(locale).patternLocale) {
        return safeFormatter(formatOptions, locale).formatRange(start, end);
    }

    return `${formatDateObjectToString(start, formatOptions, locale)} – ${formatDateObjectToString(end, formatOptions, locale)}`;
}

export function formatMonthKey(
    value: string | null | undefined,
    options: MonthKeyFormatOptions = {},
): string {
    const fallback = options.fallback ?? '--';
    if (!value) {
        return fallback;
    }

    const parsed = parseMonthKey(value);
    if (!parsed) {
        return value;
    }

    return formatDateObjectToString(parsed, {
        month: options.monthStyle ?? 'long',
        ...(options.includeYear ? { year: 'numeric' as const } : {}),
        timeZone: 'UTC',
    });
}

export function formatMonthOfYear(
    monthIndex: number,
    options: Omit<MonthKeyFormatOptions, 'includeYear'> = {},
): string {
    const fallback = options.fallback ?? '--';
    if (!Number.isInteger(monthIndex) || monthIndex < 0 || monthIndex > 11) {
        return fallback;
    }

    const parsed = new Date(Date.UTC(2026, monthIndex, 1, 12, 0, 0, 0));
    return formatDateObjectToString(parsed, {
        month: options.monthStyle ?? 'long',
        timeZone: 'UTC',
    });
}

export function formatInstant(
    value: string | null | undefined,
    options: InstantFormatOptions = {},
): string {
    const fallback = options.fallback ?? '--';
    if (!value) {
        return fallback;
    }

    const parsed = parseInstant(value);
    if (!parsed) {
        return value;
    }

    return formatDateObjectToString(parsed, {
        dateStyle: options.dateStyle ?? 'medium',
        timeStyle: options.timeStyle ?? 'short',
    });
}

export function formatDateObject(
    value: Date,
    options: Intl.DateTimeFormatOptions,
    fallback = '--',
    locale = currentLocale(),
): string {
    if (!isValidDate(value)) {
        return fallback;
    }

    return formatDateObjectToString(value, options, locale);
}

export function formatDateObjectToParts(
    value: Date,
    options: Intl.DateTimeFormatOptions,
    locale = currentLocale(),
): Intl.DateTimeFormatPart[] {
    if (!isValidDate(value)) {
        return [{ type: 'literal', value: '--' }];
    }

    return formatDateObjectToMergedParts(value, options, locale);
}
