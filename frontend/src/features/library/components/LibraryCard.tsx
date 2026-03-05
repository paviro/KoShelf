import { useMemo } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { FaHighlighter, FaPause, FaStar } from 'react-icons/fa';
import { HiSparkles } from 'react-icons/hi';
import { LuBookOpen } from 'react-icons/lu';

import {
    buildRoutePath,
    detailRouteIdForCollection,
} from '../../../app/routes/route-registry';
import { translation } from '../../../shared/i18n';
import { useLazyImageSource } from '../../../shared/lib/dom/useLazyImageSource';
import { createDetailReturnState } from '../../../shared/lib/navigation/detail-return-state';
import type { LibraryListItem } from '../api/library-data';
import { formatSeriesDisplay } from '../lib/library-detail-formatters';
import type {
    LibraryCollection,
    LibrarySectionKey,
} from '../model/library-model';

type LibraryCardProps = {
    item: LibraryListItem;
    collection: LibraryCollection;
    sectionKey: LibrarySectionKey;
};

const NOTES_COLOR_CLASSES: Record<LibrarySectionKey, string> = {
    reading:
        'bg-gradient-to-br from-blue-500 to-blue-600 border border-blue-400/30',
    abandoned:
        'bg-gradient-to-br from-purple-500 to-purple-600 border border-purple-400/30',
    completed:
        'bg-gradient-to-br from-emerald-500 to-emerald-600 border border-emerald-400/30',
    unread: 'bg-gradient-to-br from-orange-500 to-orange-600 border border-orange-400/30',
};

function toProgressPercentage(progress: number | null | undefined): number {
    if (!Number.isFinite(progress)) {
        return 0;
    }

    const percent = (progress ?? 0) * 100;
    return Math.min(100, Math.max(0, Math.round(percent)));
}

export function LibraryCard({
    item,
    collection,
    sectionKey,
}: LibraryCardProps) {
    const location = useLocation();
    const detailPath = buildRoutePath(detailRouteIdForCollection(collection), {
        id: item.id,
    });
    const primaryAuthor = item.authors[0];
    const annotationCount = item.annotation_count ?? 0;
    const progressPercentage = toProgressPercentage(item.progress_percentage);
    const seriesDisplay = formatSeriesDisplay(item.series);

    const {
        imageRef,
        resolvedSrc,
        isLoaded,
        hasError,
        shouldAnimateReveal,
        onLoad,
        onError,
    } = useLazyImageSource({
        src: item.cover_url,
    });

    const detailsAriaLabel = useMemo(() => {
        if (primaryAuthor) {
            return `${item.title} ${translation.get('by')} ${primaryAuthor}`;
        }
        return item.title;
    }, [item.title, primaryAuthor]);
    const detailReturnState = createDetailReturnState(
        location.pathname,
        location.search,
    );

    return (
        <article
            className="book-card group shadow-lg dark:shadow-none"
            data-library-item-id={item.id}
            data-library-collection={collection}
            data-library-item-title={item.title}
            data-library-item-authors={item.authors.join(', ')}
            data-library-item-series={seriesDisplay}
        >
            <Link
                to={detailPath}
                state={detailReturnState}
                className="block"
                aria-label={detailsAriaLabel}
            >
                <div className="aspect-book bg-gray-200 dark:bg-dark-700 relative overflow-hidden">
                    {!hasError && (
                        <img
                            ref={imageRef}
                            src={resolvedSrc}
                            alt={item.title}
                            className={`w-full h-full object-cover ${
                                shouldAnimateReveal
                                    ? 'transition-opacity duration-500 ease-out '
                                    : ''
                            }${isLoaded ? 'opacity-100' : 'opacity-0'}`}
                            loading="lazy"
                            onLoad={onLoad}
                            onError={onError}
                        />
                    )}

                    <div
                        className={`absolute inset-0 flex items-center justify-center text-4xl text-gray-400 dark:text-dark-500 bg-gray-300 dark:bg-gray-600 ${
                            shouldAnimateReveal
                                ? 'transition-opacity duration-300 '
                                : ''
                        }${
                            isLoaded && !hasError
                                ? 'opacity-0 pointer-events-none'
                                : 'opacity-100'
                        }`}
                    >
                        <LuBookOpen className="w-10 h-10" aria-hidden="true" />
                    </div>

                    {(sectionKey === 'reading' ||
                        sectionKey === 'abandoned') && (
                        <div
                            className="book-progress-bar progress-reading"
                            style={{ width: `${progressPercentage}%` }}
                        />
                    )}

                    {typeof item.rating === 'number' && item.rating > 0 && (
                        <div className="absolute top-2 left-2 bg-gradient-to-br from-yellow-400 to-yellow-500 text-white text-xs px-2 py-1 rounded-lg shadow-lg backdrop-blur-sm border border-yellow-300/30 flex items-center space-x-1">
                            <FaStar className="w-3 h-3" aria-hidden="true" />
                            <span className="font-medium">{item.rating}</span>
                        </div>
                    )}

                    {annotationCount > 0 && (
                        <div
                            className={`absolute top-2 right-2 ${NOTES_COLOR_CLASSES[sectionKey]} text-white text-xs px-2 py-1 rounded-lg shadow-lg backdrop-blur-sm flex items-center space-x-1`}
                        >
                            <FaHighlighter
                                className="w-3 h-3"
                                aria-hidden="true"
                            />
                            <span className="font-medium">
                                {annotationCount}
                            </span>
                        </div>
                    )}

                    {sectionKey === 'abandoned' && (
                        <div className="absolute bottom-2 left-1/2 -translate-x-1/2 bg-gradient-to-br from-gray-500 to-gray-600 text-white text-xs px-3 py-1 rounded-full shadow-lg backdrop-blur-sm border border-gray-400/30 flex items-center space-x-1 whitespace-nowrap z-10">
                            <FaPause className="w-3 h-3" aria-hidden="true" />
                            <span className="font-medium">
                                {translation.get('status.on-hold')}
                            </span>
                        </div>
                    )}

                    {sectionKey === 'unread' && (
                        <div className="absolute top-2 left-2 bg-gradient-to-br from-orange-500 to-orange-600 text-white text-xs px-2.5 py-1 rounded-lg shadow-lg backdrop-blur-sm border border-orange-400/30 flex items-center space-x-1">
                            <HiSparkles
                                className="w-3 h-3"
                                aria-hidden="true"
                            />
                            <span className="font-medium">
                                {translation.get('status.unread')}
                            </span>
                        </div>
                    )}
                </div>
            </Link>
        </article>
    );
}
