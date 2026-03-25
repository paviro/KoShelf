import { useMemo, useRef, useState } from 'react';
import { LuArrowLeft, LuBookOpen } from 'react-icons/lu';
import { Link } from 'react-router';

import {
    buildRoutePath,
    readerRouteIdForCollection,
} from '../../../app/routes/route-registry';

import { useRouteHeader } from '../../../app/shell/use-route-header';
import { api } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { useClickOutside } from '../../../shared/lib/dom/useClickOutside';
import { Button } from '../../../shared/ui/button/Button';
import { buttonVariants } from '../../../shared/ui/button/button-variants';
import { isReaderFormatSupported } from '../../reader/lib/reader-format-support';
import type { LibraryCollection } from '../model/library-model';

type LibraryDetailHeaderProps = {
    title: string;
    primaryAuthor?: string;
    collection: LibraryCollection;
    itemId: string;
    backHref: string;
    format?: string | null;
};

export function LibraryDetailHeader({
    title,
    primaryAuthor,
    collection,
    itemId,
    backHref,
    format,
}: LibraryDetailHeaderProps) {
    const [shareOpen, setShareOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    useClickOutside(dropdownRef, () => setShareOpen(false), shareOpen);

    const jsonHref = api.getItemDownloadHref(itemId);
    const jsonDownloadName = `${collection}-${itemId}.json`;
    const fileHref = api.getItemFileHref(itemId, format);
    const fileDownloadName = format
        ? `${primaryAuthor ? `${primaryAuthor} - ` : ''}${title}.${format}`
        : undefined;
    const fileLabel = format?.toUpperCase() ?? null;
    const readerHref =
        fileHref && isReaderFormatSupported(format)
            ? buildRoutePath(readerRouteIdForCollection(collection), {
                  id: itemId,
              })
            : null;

    const header = useMemo(
        () => ({
            mobileContent: (
                <div className="flex items-center space-x-3 min-w-0 flex-1">
                    <Link
                        to={backHref}
                        className="flex items-center space-x-2 text-primary-400 hover:text-primary-300 transition-colors cursor-pointer shrink-0"
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
            ),
            desktopContent: (
                <div className="min-w-0 flex-1">
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white truncate">
                        {title}
                    </h2>

                    {primaryAuthor && (
                        <p className="text-sm font-medium text-gray-500 dark:text-dark-300 truncate">
                            {translation.get('by')} {primaryAuthor}
                        </p>
                    )}
                </div>
            ),
            controls: (
                <div className="flex items-center space-x-2">
                    {readerHref && (
                        <Link
                            to={readerHref}
                            className={buttonVariants({
                                variant: 'neutral',
                                size: 'icon',
                                className: 'gap-1.5 px-3 md:px-4 w-auto',
                            })}
                            title={translation.get('open-in-reader.aria-label')}
                            aria-label={translation.get(
                                'open-in-reader.aria-label',
                            )}
                        >
                            <LuBookOpen
                                className="w-5 h-5"
                                aria-hidden="true"
                            />
                            <span className="hidden sm:inline text-sm">
                                {translation.get('open-in-reader')}
                            </span>
                        </Link>
                    )}
                    <div className="relative" ref={dropdownRef}>
                        <Button
                            id="shareDropdownButton"
                            variant="neutral"
                            size="icon"
                            className="dropdown-trigger"
                            aria-haspopup="menu"
                            aria-expanded={shareOpen}
                            aria-controls="shareDropdownMenu"
                            title={translation.get('share')}
                            aria-label={translation.get('share')}
                            onClick={() => setShareOpen((current) => !current)}
                        >
                            <svg
                                className="w-5 h-5"
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
                        </Button>

                        <div
                            id="shareDropdownMenu"
                            className={`dropdown-menu-right absolute right-0 mt-2 w-40 bg-white dark:bg-dark-800 border border-gray-200/50 dark:border-dark-700/50 rounded-lg shadow-xl z-20 overflow-hidden ${shareOpen ? '' : 'hidden'}`}
                            role="menu"
                        >
                            {fileHref && fileLabel && (
                                <a
                                    href={fileHref}
                                    download={fileDownloadName}
                                    className="block px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50 text-sm font-medium transition-colors duration-200"
                                    onClick={() => setShareOpen(false)}
                                >
                                    {fileLabel}
                                </a>
                            )}
                            <a
                                href={jsonHref}
                                download={jsonDownloadName}
                                className="block px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50 text-sm font-medium transition-colors duration-200"
                                onClick={() => setShareOpen(false)}
                            >
                                JSON
                            </a>
                        </div>
                    </div>
                </div>
            ),
        }),
        [
            backHref,
            fileDownloadName,
            fileHref,
            fileLabel,
            jsonDownloadName,
            jsonHref,
            primaryAuthor,
            readerHref,
            shareOpen,
            title,
        ],
    );

    useRouteHeader(header);
    return null;
}
