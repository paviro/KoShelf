import { Link } from 'react-router-dom';
import { LuChevronDown, LuFilter } from 'react-icons/lu';

import type { StatisticsScope } from '../api/statistics-data';
import { translation } from '../../../shared/i18n';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../../../shared/ui/dropdown/dropdown-styles';

type ScopeFilterProps = {
    showTypeFilter: boolean;
    scope: StatisticsScope;
};

export function ScopeFilter({ showTypeFilter, scope }: ScopeFilterProps) {
    if (!showTypeFilter) {
        return null;
    }

    return (
        <div className="flex items-center">
            <details className="relative">
                <summary
                    className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} list-none [&::-webkit-details-marker]:hidden text-gray-900 dark:text-white space-x-0 sm:space-x-2 sm:justify-start w-10 sm:w-auto sm:px-4`}
                >
                    <LuFilter
                        className={`sm:hidden w-5 h-5 ${scope === 'all' ? 'text-gray-600 dark:text-gray-300' : 'text-primary-500'}`}
                        aria-hidden="true"
                    />
                    <span className="hidden sm:inline font-medium">
                        {scope === 'books'
                            ? translation.get('books')
                            : scope === 'comics'
                              ? translation.get('comics')
                              : translation.get('filter.all')}
                    </span>
                    <LuChevronDown
                        className="hidden sm:block w-4 h-4 text-primary-400"
                        aria-hidden="true"
                    />
                </summary>
                <div className={`${DROPDOWN_PANEL_BASE_CLASSNAME} w-40 dark:bg-dark-800`}>
                    <Link
                        className="block w-full text-left px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50"
                        to="/statistics"
                    >
                        {translation.get('filter.all')}
                    </Link>
                    <Link
                        className="block w-full text-left px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50"
                        to="/statistics/books"
                    >
                        {translation.get('books')}
                    </Link>
                    <Link
                        className="block w-full text-left px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50"
                        to="/statistics/comics"
                    >
                        {translation.get('comics')}
                    </Link>
                </div>
            </details>
        </div>
    );
}
