import { Link } from 'react-router-dom';

import type { StatisticsScope } from '../../../shared/statistics-data-loader';
import { translation } from '../../../shared/i18n';

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
                <summary className="dropdown-trigger list-none [&::-webkit-details-marker]:hidden flex items-center justify-center sm:justify-start space-x-0 sm:space-x-2 w-10 sm:w-auto sm:px-4 h-10 bg-gray-100/50 dark:bg-dark-800/50 border border-gray-300/50 dark:border-dark-700/50 text-gray-900 dark:text-white rounded-lg cursor-pointer hover:bg-gray-200/50 dark:hover:bg-dark-700/50 text-sm md:text-base backdrop-blur-sm">
                    <svg
                        className={`sm:hidden w-5 h-5 ${scope === 'all' ? 'text-gray-600 dark:text-gray-300' : 'text-primary-500'}`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                        />
                    </svg>
                    <span className="hidden sm:inline font-medium">
                        {scope === 'books'
                            ? translation.get('books')
                            : scope === 'comics'
                              ? translation.get('comics')
                              : translation.get('filter.all')}
                    </span>
                    <svg
                        className="hidden sm:block w-4 h-4 text-primary-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            d="M19 9l-7 7-7-7"
                        />
                    </svg>
                </summary>
                <div className="dropdown-menu-right z-30 w-40 bg-white dark:bg-dark-800 border border-gray-200/50 dark:border-dark-700/50 rounded-lg shadow-xl overflow-hidden">
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
