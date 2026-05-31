import { LuDownload } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import {
    SortNewestIcon,
    SortOldestIcon,
} from '../../../shared/ui/icons/SortOrderIcons';
import { ContentScopeFilter } from '../../../shared/ui/selectors/ContentScopeFilter';
import { YearSelector } from '../../../shared/ui/selectors/YearSelector';
import type { RecapScope } from '../api/recap-data';

type RecapHeaderControlsProps = {
    showTypeFilter: boolean;
    scope: RecapScope;
    years: number[];
    selectedYear: number | null;
    onSelectYear: (year: number) => void;
    onScopeChange: (scope: RecapScope) => void;
    sortNewestFirst: boolean;
    onToggleSort: () => void;
    shareEnabled: boolean;
    onShareClick: () => void;
};

export function RecapHeaderControls({
    showTypeFilter,
    scope,
    years,
    selectedYear,
    onSelectYear,
    onScopeChange,
    sortNewestFirst,
    onToggleSort,
    shareEnabled,
    onShareClick,
}: RecapHeaderControlsProps) {
    const sortLabel = sortNewestFirst
        ? translation.get('sort-order.newest-first')
        : translation.get('sort-order.oldest-first');
    const shareLabel = translation.get('download.recap-label');

    return (
        <div className="flex items-center space-x-2 md:space-x-4">
            {years.length > 0 && (
                <YearSelector
                    years={years}
                    selectedYear={selectedYear}
                    onSelect={onSelectYear}
                    iconColorClass="text-gray-600 dark:text-gray-300 sm:text-green-400 sm:dark:text-green-400"
                    optionActiveClass="bg-green-50/50 dark:bg-dark-700/50 text-green-900 dark:text-white"
                    mobileFallback={translation.get('recap')}
                />
            )}

            <Button
                variant="neutral"
                icon={LuDownload}
                aria-label={shareLabel}
                onClick={onShareClick}
                disabled={!shareEnabled}
            />

            <Button
                variant="neutral"
                icon={sortNewestFirst ? SortNewestIcon : SortOldestIcon}
                aria-label={sortLabel}
                onClick={onToggleSort}
            />

            <ContentScopeFilter
                visible={showTypeFilter}
                value={scope}
                onChange={onScopeChange}
            />
        </div>
    );
}
