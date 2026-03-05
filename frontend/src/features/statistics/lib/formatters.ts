import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { monthKeyAt, toShortMonthKey } from './months';

function isFiniteNumber(value: number | null | undefined): value is number {
    return value !== null && value !== undefined && Number.isFinite(value);
}

export class DateFormatter {
    static parseISODate(dateStr: string): Date {
        const parsedDate = new Date(dateStr);

        if (Number.isNaN(parsedDate.getTime())) {
            return new Date();
        }

        return parsedDate;
    }

    static formatDateRange(
        startDateStr: string,
        endDateStr: string,
        monthStyle: 'short' | 'long' = 'short',
    ): string {
        const startDate = this.parseISODate(startDateStr);
        const endDate = this.parseISODate(endDateStr);

        const startDay = startDate.getDate();
        const startMonth = startDate.getMonth();
        const endDay = endDate.getDate();
        const endMonth = endDate.getMonth();
        const startYear = startDate.getFullYear();
        const endYear = endDate.getFullYear();

        const startMonthKey = monthKeyAt(startMonth);
        const endMonthKey = monthKeyAt(endMonth);
        const startMonthLabel =
            monthStyle === 'long'
                ? translation.get(startMonthKey)
                : translation.get(toShortMonthKey(startMonthKey));
        const endMonthLabel =
            monthStyle === 'long'
                ? translation.get(endMonthKey)
                : translation.get(toShortMonthKey(endMonthKey));

        if (startMonth === endMonth && startYear === endYear) {
            return `${startDay}-${endDay} ${startMonthLabel}`;
        }

        return `${startDay} ${startMonthLabel} - ${endDay} ${endMonthLabel}`;
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
