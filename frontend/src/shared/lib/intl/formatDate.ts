import { translation } from '../../i18n';

const FALLBACK_LOCALE = 'en-US';
const PLAIN_DATE_REGEX = /^(\d{4})-(\d{2})-(\d{2})$/;
const MONTH_KEY_REGEX = /^(\d{4})-(\d{2})$/;

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

    return safeFormatter(
        plainDateFormatOptions(
            parsed,
            options.yearDisplay ?? 'auto',
            undefined,
            options.monthStyle ?? 'short',
        ),
    ).format(parsed);
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

    const formatter = safeFormatter(
        plainDateFormatOptions(
            start,
            options.yearDisplay ?? 'auto',
            end,
            options.monthStyle ?? 'short',
        ),
    );

    return formatter.formatRange(start, end);
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

    return safeFormatter({
        month: options.monthStyle ?? 'long',
        ...(options.includeYear ? { year: 'numeric' as const } : {}),
        timeZone: 'UTC',
    }).format(parsed);
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
    return safeFormatter({
        month: options.monthStyle ?? 'long',
        timeZone: 'UTC',
    }).format(parsed);
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

    return safeFormatter({
        dateStyle: options.dateStyle ?? 'medium',
        timeStyle: options.timeStyle ?? 'short',
    }).format(parsed);
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

    return safeFormatter(options, locale).format(value);
}

export function formatDateObjectToParts(
    value: Date,
    options: Intl.DateTimeFormatOptions,
    locale = currentLocale(),
): Intl.DateTimeFormatPart[] {
    if (!isValidDate(value)) {
        return [{ type: 'literal', value: '--' }];
    }

    return safeFormatter(options, locale).formatToParts(value);
}
