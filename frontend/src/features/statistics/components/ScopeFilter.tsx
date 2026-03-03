import { useNavigate } from 'react-router-dom';

import type { StatisticsScope } from '../api/statistics-data';
import { ContentScopeFilter } from '../../../shared/ui/selectors/ContentScopeFilter';

type ScopeFilterProps = {
    showTypeFilter: boolean;
    scope: StatisticsScope;
};

export function ScopeFilter({ showTypeFilter, scope }: ScopeFilterProps) {
    const navigate = useNavigate();

    return (
        <ContentScopeFilter
            visible={showTypeFilter}
            value={scope}
            onChange={(nextScope) => {
                if (nextScope === 'all') {
                    navigate('/statistics');
                    return;
                }

                navigate(`/statistics/${nextScope}`);
            }}
        />
    );
}
