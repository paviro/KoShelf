import { translation } from '../../../shared/i18n';
import { formatPlainDateRange } from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import {
    joinUnitValueParts,
    type UnitValuePart,
} from '../../../shared/lib/intl/unit-value';

type UnitKey = 'units.w' | 'units.d' | 'units.h' | 'units.m';

function isFiniteNumber(value: number | null | undefined): value is number {
    return value !== null && value !== undefined && Number.isFinite(value);
}

function withUnit(value: number, unitKey: UnitKey): UnitValuePart {
    return {
        amount: formatNumber(value),
        unit: translation.get(unitKey),
    };
}

export class DateFormatter {
    static formatDateRange(
        startDateStr: string,
        endDateStr: string,
        monthStyle: 'short' | 'long' = 'short',
    ): string {
        return formatPlainDateRange(startDateStr, endDateStr, {
            monthStyle,
            yearDisplay: 'auto',
        });
    }
}

export class DataFormatter {
    static formatReadTimeParts(
        seconds: number | null | undefined,
    ): UnitValuePart[] {
        if (!isFiniteNumber(seconds)) {
            return [{ amount: '--' }];
        }

        const normalizedSeconds = Math.max(0, Math.floor(seconds));
        const hours = Math.floor(normalizedSeconds / 3600);
        const minutes = Math.floor((normalizedSeconds % 3600) / 60);

        if (hours > 0) {
            return [withUnit(hours, 'units.h'), withUnit(minutes, 'units.m')];
        }

        return [withUnit(minutes, 'units.m')];
    }

    static formatReadTime(seconds: number | null | undefined): string {
        return joinUnitValueParts(DataFormatter.formatReadTimeParts(seconds));
    }

    static formatReadTimeWithDaysParts(
        seconds: number | null | undefined,
    ): UnitValuePart[] {
        if (!isFiniteNumber(seconds)) {
            return [{ amount: '--' }];
        }

        const normalizedSeconds = Math.max(0, Math.floor(seconds));
        const totalMinutes = Math.floor(normalizedSeconds / 60);
        const totalHours = Math.floor(totalMinutes / 60);
        const days = Math.floor(totalHours / 24);
        const hours = totalHours % 24;
        const minutes = totalMinutes % 60;

        if (days > 0) {
            return [
                withUnit(days, 'units.d'),
                withUnit(hours, 'units.h'),
                withUnit(minutes, 'units.m'),
            ];
        }

        if (hours > 0) {
            return [withUnit(hours, 'units.h'), withUnit(minutes, 'units.m')];
        }

        return [withUnit(minutes, 'units.m')];
    }

    static formatMinutesParts(
        minutes: number | null | undefined,
    ): UnitValuePart[] {
        if (!isFiniteNumber(minutes)) {
            return [{ amount: '--' }];
        }

        const normalizedMinutes = Math.max(0, Math.floor(minutes));
        return [withUnit(normalizedMinutes, 'units.m')];
    }

    static formatCount(value: number | null | undefined): string {
        if (!isFiniteNumber(value)) {
            return '--';
        }

        return formatNumber(value);
    }

    static formatAvgPages(avg: number): string {
        if (!Number.isFinite(avg)) {
            return '--';
        }

        const normalized = Math.floor(avg * 10) / 10;
        return formatNumber(normalized, {
            minimumFractionDigits: 1,
            maximumFractionDigits: 1,
        });
    }
}
