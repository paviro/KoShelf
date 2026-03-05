import type { StatisticsScope } from '../api/statistics-data';
import { ContentScopeFilter } from '../../../shared/ui/selectors/ContentScopeFilter';

type ScopeFilterProps = {
    showTypeFilter: boolean;
    scope: StatisticsScope;
    onScopeChange: (scope: StatisticsScope) => void;
};

export function ScopeFilter({
    showTypeFilter,
    scope,
    onScopeChange,
}: ScopeFilterProps) {
    return (
        <ContentScopeFilter
            visible={showTypeFilter}
            value={scope}
            onChange={onScopeChange}
        />
    );
}
