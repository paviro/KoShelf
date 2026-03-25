import { translation } from '../../i18n';
import { formatNumber } from './formatNumber';
import { joinUnitValueParts, type UnitValuePart } from './unit-value';

type FormatDurationOptions = {
    includeDays?: boolean;
    includeSeconds?: boolean;
};

export function formatDurationParts(
    seconds: number | null | undefined,
    options?: FormatDurationOptions,
): UnitValuePart[] {
    if (
        typeof seconds !== 'number' ||
        !Number.isFinite(seconds) ||
        seconds <= 0
    ) {
        return [{ amount: '0', unit: translation.get('units.s') }];
    }

    const normalizedSeconds = Math.max(0, Math.floor(seconds));
    const totalMinutes = Math.floor(normalizedSeconds / 60);

    if (options?.includeSeconds) {
        if (normalizedSeconds < 60) {
            return [
                {
                    amount: formatNumber(normalizedSeconds),
                    unit: translation.get('units.s'),
                },
            ];
        }

        if (normalizedSeconds < 3600) {
            const minutes = Math.floor(normalizedSeconds / 60);
            const secs = normalizedSeconds % 60;
            const parts: UnitValuePart[] = [
                {
                    amount: formatNumber(minutes),
                    unit: translation.get('units.m'),
                },
            ];
            if (secs > 0) {
                parts.push({
                    amount: formatNumber(secs),
                    unit: translation.get('units.s'),
                });
            }
            return parts;
        }

        const hours = Math.floor(normalizedSeconds / 3600);
        const minutes = Math.floor((normalizedSeconds % 3600) / 60);
        const parts: UnitValuePart[] = [
            {
                amount: formatNumber(hours),
                unit: translation.get('units.h'),
            },
        ];
        if (minutes > 0) {
            parts.push({
                amount: formatNumber(minutes),
                unit: translation.get('units.m'),
            });
        }
        return parts;
    }

    if (options?.includeDays) {
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

    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    if (hours > 0) {
        return [
            { amount: formatNumber(hours), unit: translation.get('units.h') },
            {
                amount: formatNumber(minutes),
                unit: translation.get('units.m'),
            },
        ];
    }

    return [
        {
            amount: formatNumber(totalMinutes),
            unit: translation.get('units.m'),
        },
    ];
}

export function formatDuration(
    seconds: number | null | undefined,
    options?: FormatDurationOptions,
): string {
    return joinUnitValueParts(formatDurationParts(seconds, options));
}
