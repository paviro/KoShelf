import { useMemo, useRef, useState } from 'react';
import {
    LuCalendarDays,
    LuChevronDown,
    LuChevronLeft,
    LuChevronRight,
} from 'react-icons/lu';

import type { StatisticsIndexWeek } from '../api/statistics-data';
import { DateFormatter } from '../lib/formatters';
import { getWeekYearOrder } from '../model/statistics-model';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../../../shared/ui/dropdown/dropdown-styles';
import { DropdownPortal } from '../../../shared/ui/dropdown/DropdownPortal';

type WeekSelectorProps = {
    weeks: StatisticsIndexWeek[];
    selectedWeekKey: string | null;
    onSelect: (weekKey: string) => void;
};

export function WeekSelector({
    weeks,
    selectedWeekKey,
    onSelect,
}: WeekSelectorProps) {
    const triggerRef = useRef<HTMLButtonElement>(null);
    const [open, setOpen] = useState(false);
    const [view, setView] = useState<'years' | 'weeks'>('years');
    const yearOrder = useMemo(() => getWeekYearOrder(weeks), [weeks]);

    const selectedWeek = useMemo(() => {
        if (!weeks.length) {
            return null;
        }
        return (
            weeks.find((week) => week.week_key === selectedWeekKey) ?? weeks[0]
        );
    }, [selectedWeekKey, weeks]);

    const [prevSelectedWeek, setPrevSelectedWeek] = useState(selectedWeek);
    const [selectedYear, setSelectedYear] = useState<string | null>(
        selectedWeek
            ? selectedWeek.start_date.substring(0, 4)
            : (yearOrder[0] ?? null),
    );

    if (selectedWeek !== prevSelectedWeek) {
        setPrevSelectedWeek(selectedWeek);
        if (selectedWeek) {
            setSelectedYear(selectedWeek.start_date.substring(0, 4));
        }
    }

    const selectedText = selectedWeek
        ? DateFormatter.formatDateRange(
              selectedWeek.start_date,
              selectedWeek.end_date,
          )
        : 'No weeks available';

    const weeksForSelectedYear = weeks.filter(
        (week) => week.start_date.substring(0, 4) === (selectedYear ?? ''),
    );

    return (
        <>
            <button
                ref={triggerRef}
                id="weekSelectorWrapper"
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-controls="weekOptions"
                onClick={() => {
                    if (!open) {
                        setView('weeks');
                    }
                    setOpen((current) => !current);
                }}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} w-10 sm:w-auto sm:px-4`}
            >
                <div className="flex items-center space-x-0 sm:space-x-3 min-w-0">
                    <LuCalendarDays
                        className="w-5 h-5 text-gray-600 dark:text-gray-300 sm:text-primary-400 sm:dark:text-primary-400"
                        aria-hidden="true"
                    />
                    <span
                        id="selectedWeekText"
                        className="hidden sm:inline text-gray-900 dark:text-white font-medium text-sm truncate"
                    >
                        {selectedWeek ? (
                            <span className="font-bold">{selectedText}</span>
                        ) : (
                            'No weeks available'
                        )}
                    </span>
                </div>
                <LuChevronDown
                    id="dropdownArrow"
                    className="hidden sm:block w-4 h-4 text-gray-400 dark:text-dark-400 transition-transform duration-200 ml-2"
                    aria-hidden="true"
                />
            </button>

            <DropdownPortal
                triggerRef={triggerRef}
                open={open}
                onClose={() => setOpen(false)}
                closeOnScroll
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} w-56 sm:w-64 max-w-[calc(100vw-1rem)]`}
            >
                <div
                    id="weekYearList"
                    className={`max-h-56 overflow-y-auto ${view === 'years' ? '' : 'hidden'}`}
                >
                    {yearOrder.map((year) => {
                        const active = selectedYear === year;
                        return (
                            <button
                                key={year}
                                type="button"
                                className={`week-year-option w-full text-left px-4 py-2.5 cursor-pointer hover:bg-gray-100/60 dark:hover:bg-dark-700/60 transition-colors duration-200 ${
                                    active
                                        ? 'bg-primary-50/50 dark:bg-dark-700/50 text-primary-900 dark:text-white'
                                        : 'text-gray-600 dark:text-dark-200 hover:text-gray-900 dark:hover:text-white'
                                }`}
                                data-week-year={year}
                                onClick={() => {
                                    setSelectedYear(year);
                                    setView('weeks');
                                }}
                            >
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center">
                                        <LuCalendarDays
                                            className="w-4 h-4 text-primary-400 mr-2"
                                            aria-hidden="true"
                                        />
                                        <span className="font-semibold">
                                            {year}
                                        </span>
                                    </div>
                                    <LuChevronRight
                                        className="w-4 h-4 text-gray-400 dark:text-dark-400"
                                        aria-hidden="true"
                                    />
                                </div>
                            </button>
                        );
                    })}
                </div>

                <div
                    id="weekYearWeeksView"
                    className={view === 'weeks' ? '' : 'hidden'}
                >
                    <div className="flex items-center justify-between px-3 py-2 border-b border-gray-200/60 dark:border-dark-700/60 bg-gray-50/50 dark:bg-dark-900/30">
                        <button
                            id="weekYearBackButton"
                            type="button"
                            className="inline-flex items-center justify-center w-8 h-8 rounded-md text-gray-500 dark:text-dark-300 hover:bg-gray-200/70 dark:hover:bg-dark-700/70 transition-colors duration-200"
                            aria-label="Back to years"
                            onClick={() => setView('years')}
                        >
                            <LuChevronLeft
                                className="w-4 h-4"
                                aria-hidden="true"
                            />
                        </button>
                        <span
                            id="weekYearTitle"
                            className="text-sm font-semibold text-gray-900 dark:text-white"
                        >
                            {selectedYear}
                        </span>
                        <span className="w-8 h-8"></span>
                    </div>

                    <div
                        id="weekYearWeekList"
                        className="max-h-56 overflow-y-auto"
                    >
                        {weeksForSelectedYear.map((week) => {
                            const active =
                                week.week_key === selectedWeek?.week_key;
                            return (
                                <button
                                    key={week.week_key}
                                    type="button"
                                    className={`week-option w-full text-left px-4 py-2.5 cursor-pointer hover:bg-gray-100/60 dark:hover:bg-dark-700/60 transition-colors duration-200 ${
                                        active
                                            ? 'bg-primary-50/50 dark:bg-dark-700/50 text-primary-900 dark:text-white'
                                            : 'text-gray-600 dark:text-dark-200 hover:text-gray-900 dark:hover:text-white'
                                    }`}
                                    data-week-key={week.week_key}
                                    data-start-date={week.start_date}
                                    data-end-date={week.end_date}
                                    data-week-year={week.start_date.substring(
                                        0,
                                        4,
                                    )}
                                    onClick={() => {
                                        onSelect(week.week_key);
                                        setSelectedYear(
                                            week.start_date.substring(0, 4),
                                        );
                                        setOpen(false);
                                    }}
                                >
                                    <div className="flex items-center">
                                        <LuCalendarDays
                                            className="w-4 h-4 text-primary-400 mr-2 shrink-0"
                                            aria-hidden="true"
                                        />
                                        <span className="week-date-display">
                                            {DateFormatter.formatDateRange(
                                                week.start_date,
                                                week.end_date,
                                                'long',
                                            )}
                                        </span>
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                </div>
            </DropdownPortal>
        </>
    );
}
