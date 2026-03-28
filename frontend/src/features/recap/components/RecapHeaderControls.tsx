import { LuDownload } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
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

function SortNewestIcon(props: React.SVGAttributes<SVGElement>) {
    return (
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" {...props}>
            <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 7h8M4 12h8M4 17h5M18 6v12M15 15l3 3 3-3"
            />
        </svg>
    );
}

function SortOldestIcon(props: React.SVGAttributes<SVGElement>) {
    return (
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" {...props}>
            <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 7h5M4 12h8M4 17h8M18 18V6M15 9l3-3 3 3"
            />
        </svg>
    );
}

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
