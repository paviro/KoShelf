import type { RefObject } from 'react';
import { LuSearch, LuX } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import type { LibraryFilterValue } from '../model/library-model';
import { LibraryStatusFilter } from './LibraryStatusFilter';

type LibraryHeaderProps = {
    title: string;
    searchTerm: string;
    onSearchTermChange: (value: string) => void;
    filterValue: LibraryFilterValue;
    filterOptions: readonly LibraryFilterValue[];
    onFilterChange: (value: LibraryFilterValue) => void;
    mobileSearchOpen: boolean;
    onOpenMobileSearch: () => void;
    onCloseMobileSearch: () => void;
    desktopSearchInputRef: RefObject<HTMLInputElement | null>;
    mobileSearchInputRef: RefObject<HTMLInputElement | null>;
};

export function LibraryHeader({
    title,
    searchTerm,
    onSearchTermChange,
    filterValue,
    filterOptions,
    onFilterChange,
    mobileSearchOpen,
    onOpenMobileSearch,
    onCloseMobileSearch,
    desktopSearchInputRef,
    mobileSearchInputRef,
}: LibraryHeaderProps) {
    return (
        <header className="fixed top-0 left-0 right-0 lg:left-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-sm border-b border-gray-200/50 dark:border-dark-700/50 px-4 md:px-6 h-[70px] md:h-[80px] z-40">
            <div className="flex items-center justify-between h-full">
                <div
                    className={`lg:hidden flex items-center ${mobileSearchOpen ? 'hidden' : ''}`}
                    aria-hidden={mobileSearchOpen}
                >
                    <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                        {title}
                    </h1>
                </div>

                <div className={`lg:hidden flex-1 mr-3 ${mobileSearchOpen ? '' : 'hidden'}`}>
                    <input
                        ref={mobileSearchInputRef}
                        type="text"
                        value={searchTerm}
                        placeholder={translation.get('search-placeholder')}
                        aria-label={translation.get('search.aria-label')}
                        className="w-full bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg px-4 py-2 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-none focus:ring-2 focus:ring-primary-500/50 shadow-sm text-sm backdrop-blur-sm"
                        onChange={(event) => onSearchTermChange(event.target.value)}
                    />
                </div>

                <h2 className="hidden lg:block text-2xl font-bold text-gray-900 dark:text-white">
                    {title}
                </h2>

                <div className="flex items-center space-x-3 md:space-x-4">
                    <div className="relative hidden sm:block">
                        <input
                            ref={desktopSearchInputRef}
                            type="text"
                            value={searchTerm}
                            placeholder={translation.get('search-placeholder')}
                            aria-label={translation.get('search.aria-label')}
                            className="bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg px-4 py-2 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-none focus:ring-2 focus:ring-primary-500/50 focus:border-primary-500/50 transition-all duration-200 shadow-sm w-40 sm:w-48 md:w-64 text-sm md:text-base backdrop-blur-sm"
                            onChange={(event) => onSearchTermChange(event.target.value)}
                        />
                    </div>

                    {!mobileSearchOpen && (
                        <button
                            type="button"
                            className="sm:hidden w-10 h-10 flex items-center justify-center bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors backdrop-blur-sm"
                            title={translation.get('search.aria-label')}
                            aria-label={translation.get('search.aria-label')}
                            onClick={onOpenMobileSearch}
                        >
                            <LuSearch
                                className="w-5 h-5 text-gray-600 dark:text-gray-300"
                                aria-hidden="true"
                            />
                        </button>
                    )}

                    {mobileSearchOpen && (
                        <button
                            type="button"
                            className="sm:hidden w-10 h-10 flex items-center justify-center bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors backdrop-blur-sm"
                            title={translation.get('close-search.aria-label')}
                            aria-label={translation.get('close-search.aria-label')}
                            onClick={onCloseMobileSearch}
                        >
                            <LuX
                                className="w-5 h-5 text-gray-600 dark:text-gray-300"
                                aria-hidden="true"
                            />
                        </button>
                    )}

                    <div className={`${mobileSearchOpen ? 'hidden sm:flex' : 'flex'} items-center`}>
                        <LibraryStatusFilter
                            value={filterValue}
                            options={filterOptions}
                            onChange={onFilterChange}
                        />
                    </div>
                </div>
            </div>
        </header>
    );
}
