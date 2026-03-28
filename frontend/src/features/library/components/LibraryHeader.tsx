import { useMemo, type RefObject } from 'react';
import { LuSearch, LuX } from 'react-icons/lu';

import { useRouteHeader } from '../../../app/shell/use-route-header';
import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
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
    const header = useMemo(
        () => ({
            mobileContent: (
                <>
                    <div
                        className={`lg:hidden flex items-center min-w-0 ${mobileSearchOpen ? 'hidden' : ''}`}
                        aria-hidden={mobileSearchOpen}
                    >
                        <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                            {title}
                        </h1>
                    </div>

                    <div
                        className={`lg:hidden flex-1 mr-3 ${mobileSearchOpen ? '' : 'hidden'}`}
                    >
                        <input
                            ref={mobileSearchInputRef}
                            type="text"
                            value={searchTerm}
                            placeholder={translation.get('search-placeholder')}
                            aria-label={translation.get('search.aria-label')}
                            className="w-full bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg px-4 py-2 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-hidden focus:ring-2 focus:ring-primary-500/50 shadow-xs text-sm backdrop-blur-xs"
                            onChange={(event) =>
                                onSearchTermChange(event.target.value)
                            }
                        />
                    </div>
                </>
            ),
            desktopContent: (
                <h2 className="text-2xl font-bold text-gray-900 dark:text-white truncate">
                    {title}
                </h2>
            ),
            controls: (
                <div className="flex items-center space-x-3 md:space-x-4">
                    <div className="relative hidden sm:block">
                        <input
                            ref={desktopSearchInputRef}
                            type="text"
                            value={searchTerm}
                            placeholder={translation.get('search-placeholder')}
                            aria-label={translation.get('search.aria-label')}
                            className="bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg px-4 py-2 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-hidden focus:ring-2 focus:ring-primary-500/50 focus:border-primary-500/50 transition-all duration-200 shadow-xs w-40 sm:w-48 md:w-64 text-sm md:text-base backdrop-blur-xs"
                            onChange={(event) =>
                                onSearchTermChange(event.target.value)
                            }
                        />
                    </div>

                    {!mobileSearchOpen && (
                        <Button
                            variant="neutral"
                            className="sm:hidden"
                            icon={LuSearch}
                            aria-label={translation.get('search.aria-label')}
                            onClick={onOpenMobileSearch}
                        />
                    )}

                    {mobileSearchOpen && (
                        <Button
                            variant="neutral"
                            className="sm:hidden"
                            icon={LuX}
                            aria-label={translation.get(
                                'close-search.aria-label',
                            )}
                            onClick={onCloseMobileSearch}
                        />
                    )}

                    <div
                        className={`${mobileSearchOpen ? 'hidden sm:flex' : 'flex'} items-center`}
                    >
                        <LibraryStatusFilter
                            value={filterValue}
                            options={filterOptions}
                            onChange={onFilterChange}
                        />
                    </div>
                </div>
            ),
        }),
        [
            desktopSearchInputRef,
            filterOptions,
            filterValue,
            mobileSearchInputRef,
            mobileSearchOpen,
            onCloseMobileSearch,
            onFilterChange,
            onOpenMobileSearch,
            onSearchTermChange,
            searchTerm,
            title,
        ],
    );

    useRouteHeader(header);
    return null;
}
