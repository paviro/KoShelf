import { useMemo } from 'react';

import type { ScopeValue } from '../../api';
import { translation } from '../../i18n';
import { FilterDropdown, type FilterDropdownOption } from './FilterDropdown';

type ContentScopeFilterProps = {
    visible: boolean;
    value: ScopeValue;
    onChange: (scope: ScopeValue) => void;
};

export function ContentScopeFilter({
    visible,
    value,
    onChange,
}: ContentScopeFilterProps) {
    const options = useMemo<FilterDropdownOption<ScopeValue>[]>(
        () => [
            { value: 'all', label: translation.get('filter.all') },
            { value: 'books', label: translation.get('books') },
            { value: 'comics', label: translation.get('comics') },
        ],
        [],
    );

    if (!visible) {
        return null;
    }

    return (
        <FilterDropdown
            value={value}
            options={options}
            onChange={onChange}
            ariaLabel={translation.get('filter.aria-label')}
        />
    );
}
