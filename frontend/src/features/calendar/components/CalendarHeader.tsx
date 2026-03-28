import { useMemo } from 'react';
import { LuChevronLeft, LuChevronRight } from 'react-icons/lu';

import { useRouteHeader } from '../../../app/shell/use-route-header';
import type { ScopeValue } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import { ContentScopeFilter } from '../../../shared/ui/selectors/ContentScopeFilter';

type CalendarHeaderProps = {
    monthYearParts: Intl.DateTimeFormatPart[];
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
    monthYearParts,
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
    const header = useMemo(() => {
        const titleContent = monthYearParts.map((part, index) => {
            if (part.type === 'month') {
                return (
                    <button
                        key={`${part.type}-${index}`}
                        type="button"
                        className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors cursor-pointer"
                        onClick={onOpenMonthPicker}
                    >
                        {part.value}
                    </button>
                );
            }

            if (part.type === 'year') {
                return (
                    <button
                        key={`${part.type}-${index}`}
                        type="button"
                        className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors cursor-pointer"
                        onClick={onOpenYearPicker}
                    >
                        {part.value}
                    </button>
                );
            }

            return <span key={`${part.type}-${index}`}>{part.value}</span>;
        });

        return {
            mobileContent: (
                <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                    {titleContent}
                </h1>
            ),
            desktopContent: (
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                    {titleContent}
                </h2>
            ),
            controls: (
                <div className="flex items-center space-x-2 md:space-x-4">
                    <div className="flex items-center space-x-1">
                        <Button
                            variant="neutral"
                            icon={LuChevronLeft}
                            aria-label={translation.get(
                                'previous-month.aria-label',
                            )}
                            onClick={onPreviousMonth}
                        />
                        <Button
                            variant="neutral"
                            icon={LuChevronRight}
                            aria-label={translation.get(
                                'next-month.aria-label',
                            )}
                            onClick={onNextMonth}
                        />
                    </div>

                    <Button
                        variant="neutral"
                        className="hidden sm:flex md:text-base"
                        onClick={onToday}
                        disabled={todayDisabled}
                    >
                        {translation.get('today')}
                    </Button>

                    <ContentScopeFilter
                        visible={showTypeFilter}
                        value={scope}
                        onChange={onScopeChange}
                    />
                </div>
            ),
        };
    }, [
        monthYearParts,
        onNextMonth,
        onOpenMonthPicker,
        onOpenYearPicker,
        onPreviousMonth,
        onScopeChange,
        onToday,
        scope,
        showTypeFilter,
        todayDisabled,
    ]);

    useRouteHeader(header);
    return null;
}
