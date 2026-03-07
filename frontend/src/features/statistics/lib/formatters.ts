import { translation } from '../../../shared/i18n';
import { formatPlainDateRange } from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';

function isFiniteNumber(value: number | null | undefined): value is number {
    return value !== null && value !== undefined && Number.isFinite(value);
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
    static formatReadTime(seconds: number | null | undefined): string {
        if (!isFiniteNumber(seconds)) {
            return '--';
        }

        const normalizedSeconds = Math.max(0, Math.floor(seconds));
        const hours = Math.floor(normalizedSeconds / 3600);
        const minutes = Math.floor((normalizedSeconds % 3600) / 60);

        if (hours > 0) {
            return `${formatNumber(hours)}${translation.get('units.h')} ${formatNumber(minutes)}${translation.get('units.m')}`;
        }

        return `${formatNumber(minutes)}${translation.get('units.m')}`;
    }

    static formatReadTimeWithDays(seconds: number | null | undefined): string {
        if (!isFiniteNumber(seconds)) {
            return '--';
        }

        const normalizedSeconds = Math.max(0, Math.floor(seconds));
        const totalMinutes = Math.floor(normalizedSeconds / 60);
        const totalHours = Math.floor(totalMinutes / 60);
        const days = Math.floor(totalHours / 24);
        const hours = totalHours % 24;
        const minutes = totalMinutes % 60;

        if (days > 0) {
            return `${formatNumber(days)}${translation.get('units.d')} ${formatNumber(hours)}${translation.get('units.h')} ${formatNumber(minutes)}${translation.get('units.m')}`;
        }

        if (hours > 0) {
            return `${formatNumber(hours)}${translation.get('units.h')} ${formatNumber(minutes)}${translation.get('units.m')}`;
        }

        return `${formatNumber(minutes)}${translation.get('units.m')}`;
    }

    static formatMinutes(minutes: number | null | undefined): string {
        if (!isFiniteNumber(minutes)) {
            return '--';
        }

        const normalizedMinutes = Math.max(0, Math.floor(minutes));
        return `${formatNumber(normalizedMinutes)}${translation.get('units.m')}`;
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
