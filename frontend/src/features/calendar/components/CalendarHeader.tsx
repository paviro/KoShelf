import { useMemo } from 'react';
import { LuChevronLeft, LuChevronRight } from 'react-icons/lu';

import { useRouteHeader } from '../../../app/shell/route-header';
import type { ScopeValue } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { ContentScopeFilter } from '../../../shared/ui/selectors/ContentScopeFilter';

type CalendarHeaderProps = {
    monthLabel: string;
    yearLabel: string;
    scope: ScopeValue;
    showTypeFilter: boolean;
    onScopeChange: (scope: ScopeValue) => void;
    onPreviousMonth: () => void;
    onNextMonth: () => void;
    onToday: () => void;
    onOpenMonthPicker: () => void;
    onOpenYearPicker: () => void;
    todayDisabled: boolean;
};

export function CalendarHeader({
    monthLabel,
    yearLabel,
    scope,
    showTypeFilter,
    onScopeChange,
    onPreviousMonth,
    onNextMonth,
    onToday,
    onOpenMonthPicker,
    onOpenYearPicker,
    todayDisabled,
}: CalendarHeaderProps) {
    const titleContent = (
        <>
            <button
                type="button"
                className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors cursor-pointer"
                onClick={onOpenMonthPicker}
            >
                {monthLabel}
            </button>
            <button
                type="button"
                className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors cursor-pointer"
                onClick={onOpenYearPicker}
            >
                {yearLabel}
            </button>
        </>
    );
    const header = useMemo(
        () => ({
            mobileContent: (
                <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate flex items-center gap-[0.45rem]">
                    {titleContent}
                </h1>
            ),
            desktopContent: (
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white items-center gap-[0.45rem] flex">
                    {titleContent}
                </h2>
            ),
            controls: (
                <div className="flex items-center space-x-2 md:space-x-4">
                    <div className="flex items-center space-x-1">
                        <button
                            type="button"
                            className="px-3 md:px-4 py-2 h-10 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 text-gray-900 dark:text-white rounded-lg transition-colors duration-200 flex items-center justify-center backdrop-blur-sm"
                            title={translation.get('previous-month.aria-label')}
                            aria-label={translation.get(
                                'previous-month.aria-label',
                            )}
                            onClick={onPreviousMonth}
                        >
                            <LuChevronLeft
                                className="w-4 h-4"
                                aria-hidden="true"
                            />
                        </button>
                        <button
                            type="button"
                            className="px-3 md:px-4 py-2 h-10 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 text-gray-900 dark:text-white rounded-lg transition-colors duration-200 flex items-center justify-center backdrop-blur-sm"
                            title={translation.get('next-month.aria-label')}
                            aria-label={translation.get(
                                'next-month.aria-label',
                            )}
                            onClick={onNextMonth}
                        >
                            <LuChevronRight
                                className="w-4 h-4"
                                aria-hidden="true"
                            />
                        </button>
                    </div>

                    <button
                        type="button"
                        className={`hidden sm:flex px-3 md:px-4 py-2 h-10 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 rounded-lg font-medium text-sm md:text-base transition-colors duration-200 items-center justify-center backdrop-blur-sm ${
                            todayDisabled
                                ? 'bg-gray-100 dark:bg-dark-800 text-gray-400 dark:text-dark-400 cursor-not-allowed'
                                : 'bg-primary-600 hover:bg-primary-700 text-white'
                        }`}
                        onClick={onToday}
                        disabled={todayDisabled}
                    >
                        {translation.get('today')}
                    </button>

                    <ContentScopeFilter
                        visible={showTypeFilter}
                        value={scope}
                        onChange={onScopeChange}
                    />
                </div>
            ),
        }),
        [
            monthLabel,
            onNextMonth,
            onOpenMonthPicker,
            onOpenYearPicker,
            onPreviousMonth,
            onScopeChange,
            onToday,
            scope,
            showTypeFilter,
            todayDisabled,
            yearLabel,
        ],
    );

    useRouteHeader(header);
    return null;
}
