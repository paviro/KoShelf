import { useRef, useState } from 'react';
import { LuArrowLeft } from 'react-icons/lu';
import { Link } from 'react-router-dom';

import { translation } from '../../../shared/i18n';
import { useClickOutside } from '../../../shared/lib/dom/useClickOutside';
import type { LibraryCollection } from '../model/library-model';

type LibraryDetailHeaderProps = {
    title: string;
    primaryAuthor?: string;
    collection: LibraryCollection;
    itemId: string;
    backHref: string;
};

export function LibraryDetailHeader({
    title,
    primaryAuthor,
    collection,
    itemId,
    backHref,
}: LibraryDetailHeaderProps) {
    const [shareOpen, setShareOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    useClickOutside(dropdownRef, () => setShareOpen(false), shareOpen);

    const markdownHref = `/${collection}/${itemId}/details.md`;
    const jsonHref = `/data/${collection}/${itemId}.json`;

    return (
        <header className="fixed top-0 left-0 right-0 lg:left-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-sm border-b border-gray-200/50 dark:border-dark-700/50 px-4 md:px-6 h-[70px] md:h-[80px] z-40 flex items-center justify-between">
            <div className="flex items-center h-full min-w-0 flex-1">
                <div className="lg:hidden flex items-center space-x-3 min-w-0 flex-1">
                    <Link
                        to={backHref}
                        className="flex items-center space-x-2 text-primary-400 hover:text-primary-300 transition-colors cursor-pointer flex-shrink-0"
                        title={translation.get('go-back.aria-label')}
                        aria-label={translation.get('go-back.aria-label')}
                    >
                        <LuArrowLeft className="w-6 h-6" aria-hidden="true" />
                    </Link>

                    <div className="h-8 w-px bg-gray-200 dark:bg-dark-700 mx-3 md:mx-6"></div>

                    <div className="min-w-0 flex-1">
                        <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                            {title}
                        </h1>

                        {primaryAuthor && (
                            <p className="text-xs text-gray-500 dark:text-dark-300 truncate">
                                {translation.get('by')} {primaryAuthor}
                            </p>
                        )}
                    </div>
                </div>

                <div className="hidden lg:block min-w-0 flex-1">
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white truncate">
                        {title}
                    </h2>

                    {primaryAuthor && (
                        <p className="text-sm text-gray-500 dark:text-dark-300 truncate">
                            {translation.get('by')} {primaryAuthor}
                        </p>
                    )}
                </div>
            </div>

            <div className="flex items-center space-x-2">
                <div className="relative" ref={dropdownRef}>
                    <button
                        id="shareDropdownButton"
                        type="button"
                        aria-haspopup="menu"
                        aria-expanded={shareOpen}
                        aria-controls="shareDropdownMenu"
                        title={translation.get('share')}
                        aria-label={translation.get('share')}
                        className="dropdown-trigger p-2 bg-gray-100/50 dark:bg-dark-800/50 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors"
                        onClick={() => setShareOpen((current) => !current)}
                    >
                        <svg
                            className="w-5 h-5 text-primary-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2"
                            />
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M7 10l5 5 5-5"
                            />
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M12 4v12"
                            />
                        </svg>
                    </button>

                    <div
                        id="shareDropdownMenu"
                        className={`dropdown-menu-right absolute right-0 mt-2 w-40 bg-white dark:bg-dark-800 border border-gray-200/50 dark:border-dark-700/50 rounded-lg shadow-xl z-20 overflow-hidden ${shareOpen ? '' : 'hidden'}`}
                        role="menu"
                    >
                        <a
                            href={markdownHref}
                            download
                            className="block px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50 text-sm transition-colors duration-200"
                            onClick={() => setShareOpen(false)}
                        >
                            Markdown
                        </a>
                        <a
                            href={jsonHref}
                            download
                            className="block px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50 text-sm transition-colors duration-200"
                            onClick={() => setShareOpen(false)}
                        >
                            JSON
                        </a>
                    </div>
                </div>
            </div>
        </header>
    );
}
