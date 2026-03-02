import { translation } from './i18n.js';
import { monthKeyAt, toShortMonthKey } from './statistics-months.js';

export class DateFormatter {
    // Parse ISO date string and return a Date object
    static parseISODate(dateStr: string): Date {
        try {
            return new Date(dateStr);
        } catch {
            console.error('Error parsing date:', dateStr);
            return new Date(); // Return current date as fallback
        }
    }

    // Format date as "D Month" (e.g. "17 March")
    static formatDateNice(dateObj: Date): string {
        return `${dateObj.getDate()} ${translation.get(monthKeyAt(dateObj.getMonth()))}`;
    }

    // Format a date range nicely (e.g. "17-23 March" or "28 Feb - 5 March")
    static formatDateRange(startDateStr: string, endDateStr: string): string {
        const startDate = this.parseISODate(startDateStr);
        const endDate = this.parseISODate(endDateStr);

        const startDay = startDate.getDate();
        const startMonth = startDate.getMonth();
        const endDay = endDate.getDate();
        const endMonth = endDate.getMonth();
        const startYear = startDate.getFullYear();
        const endYear = endDate.getFullYear();

        const startMonthLabel = translation.get(toShortMonthKey(monthKeyAt(startMonth)));
        const endMonthLabel = translation.get(toShortMonthKey(monthKeyAt(endMonth)));

        // If same month
        if (startMonth === endMonth && startYear === endYear) {
            return `${startDay}-${endDay} ${startMonthLabel}`;
        }

        // Different months
        return `${startDay} ${startMonthLabel} - ${endDay} ${endMonthLabel}`;
    }
}

export class DataFormatter {
    // Format read time from seconds to hours and minutes
    static formatReadTime(seconds: number | null | undefined): string {
        if (seconds === null || seconds === undefined) {
            return '--';
        }
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }

        return `${minutes}m`;
    }

    // Format average pages with one decimal place
    static formatAvgPages(avg: number): string {
        return (Math.floor(avg * 10) / 10).toFixed(1);
    }
}
