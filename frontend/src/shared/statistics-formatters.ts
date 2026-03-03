import { translation } from './i18n';
import { monthKeyAt, toShortMonthKey } from './statistics-months';

export class DateFormatter {
    static parseISODate(dateStr: string): Date {
        try {
            return new Date(dateStr);
        } catch {
            return new Date();
        }
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
        if (seconds === null || seconds === undefined) {
            return '--';
        }

        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (hours > 0) {
            return `${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
        }

        return `${minutes}${translation.get('units.m')}`;
    }

    static formatReadTimeWithDays(seconds: number | null | undefined): string {
        if (seconds === null || seconds === undefined) {
            return '--';
        }

        const totalMinutes = Math.floor(seconds / 60);
        const totalHours = Math.floor(totalMinutes / 60);
        const days = Math.floor(totalHours / 24);
        const hours = totalHours % 24;
        const minutes = totalMinutes % 60;

        if (days > 0) {
            return `${days}${translation.get('units.d')} ${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
        }

        if (hours > 0) {
            return `${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
        }

        return `${minutes}${translation.get('units.m')}`;
    }

    static formatAvgPages(avg: number): string {
        return (Math.floor(avg * 10) / 10).toFixed(1);
    }
}
