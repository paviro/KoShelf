import { useMemo } from 'react';
import { Link, useLocation } from 'react-router';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import {
    LuCalendarDays,
    LuClock3,
    LuFileText,
    LuQuote,
    LuStar,
} from 'react-icons/lu';

import {
    buildRoutePath,
    detailRouteIdForContentType,
} from '../../../app/routes/route-registry';
import { translation } from '../../../shared/i18n';
import { useLazyImageSource } from '../../../shared/lib/dom/useLazyImageSource';
import { formatDurationParts } from '../../../shared/lib/intl/formatDuration';
import { createDetailReturnState } from '../../../shared/lib/navigation/detail-return-state';
import { MetricCardCompact } from '../../../shared/ui/cards/MetricCardCompact';
import { MetricCardUnitValue } from '../../../shared/ui/cards/MetricCardUnitValue';
import type { CompletionItem } from '../api/recap-data';
import {
    buildStarDisplay,
    formatRecapDateRange,
    resolveRecapSearchBasePath,
} from '../lib/recap-formatters';

type RecapItemCardProps = {
    item: CompletionItem;
};

export function RecapItemCard({ item }: RecapItemCardProps) {
    const location = useLocation();
    const detailPath = useMemo(() => {
        const itemId = item.item_id?.trim() ?? '';
        if (!itemId) {
            return null;
        }

        if (item.content_type !== 'book' && item.content_type !== 'comic') {
            return null;
        }

        return buildRoutePath(detailRouteIdForContentType(item.content_type), {
            id: itemId,
        });
    }, [item.content_type, item.item_id]);
    const coverUrl = item.item_cover?.trim() || null;
    const {
        imageRef,
        resolvedSrc: resolvedCoverSrc,
        hasError: coverFailed,
        onError: onCoverError,
    } = useLazyImageSource({
        src: coverUrl ?? '',
    });
    const hasReviewNote = Boolean(item.review_note?.trim());
    const hasRating =
        typeof item.rating === 'number' && Number.isFinite(item.rating);
    const stars = buildStarDisplay(item.rating);
    const searchBasePath = resolveRecapSearchBasePath(item);
    const detailReturnState = createDetailReturnState(
        location.pathname,
        location.search,
    );
    const coverFrameClass =
        'w-full flex items-center justify-center recap-cover-max';
    const coverImageClass =
        'block max-w-full max-h-full object-contain rounded-md recap-cover-tilt';
    const fallbackFrameClass =
        'w-full flex items-center justify-center rounded-md overflow-hidden border border-gray-200 dark:border-dark-600 bg-white dark:bg-dark-900/70 recap-cover-max';
    const hasCoverImage = Boolean(coverUrl && !coverFailed);
    const fallbackCoverClass = `${fallbackFrameClass} text-gray-400 dark:text-dark-400`;

    const coverVisual = hasCoverImage ? (
        <div className={coverFrameClass}>
            <img
                ref={imageRef}
                className={coverImageClass}
                src={resolvedCoverSrc}
                alt={`Cover of ${item.title}`}
                loading="lazy"
                onError={onCoverError}
            />
        </div>
    ) : (
        <div className={fallbackCoverClass}>
            <HiOutlineBookOpen className="w-12 h-12" aria-hidden />
        </div>
    );
    const coverNode = detailPath ? (
        <Link
            to={detailPath}
            state={detailReturnState}
            className="block w-full"
        >
            {coverVisual}
        </Link>
    ) : (
        coverVisual
    );

    const titleNode = detailPath ? (
        <Link
            to={detailPath}
            state={detailReturnState}
            className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
        >
            {item.title}
        </Link>
    ) : (
        <div className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
            {item.title}
        </div>
    );

    return (
        <div
            className="relative pl-10 recap-event recap-item"
            data-content-type={item.content_type ?? 'unknown'}
        >
            <span className="recap-dot bg-primary-500"></span>
            <div className="bg-white dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/50 rounded-xl shadow-xs overflow-hidden hover:shadow-md transition-shadow duration-300">
                <div className="flex flex-col md:flex-row md:items-stretch">
                    <div className="md:w-48 bg-gray-50 dark:bg-dark-800 p-4 md:self-start flex items-center justify-center">
                        {coverNode}
                    </div>

                    <div className="md:flex-1 p-4 md:p-6 md:flex md:flex-col md:justify-center">
                        <div className="flex items-start justify-between gap-4">
                            <div className="flex-1">
                                {titleNode}
                                {item.series && (
                                    <div className="text-sm font-medium text-gray-500 dark:text-dark-300 mt-1">
                                        {item.series}
                                    </div>
                                )}
                                {item.authors.length > 0 && (
                                    <div className="text-sm font-medium text-gray-600 dark:text-dark-300">
                                        {translation.get('by')}{' '}
                                        {item.authors.map((author, index) => {
                                            const separator =
                                                index < item.authors.length - 1
                                                    ? ', '
                                                    : '';
                                            if (!detailPath) {
                                                return (
                                                    <span
                                                        key={`${author}:${index}`}
                                                    >
                                                        {author}
                                                        {separator}
                                                    </span>
                                                );
                                            }

                                            return (
                                                <span
                                                    key={`${author}:${index}`}
                                                >
                                                    <Link
                                                        to={`${searchBasePath}?search=${encodeURIComponent(author)}`}
                                                        className="text-primary-600 dark:text-primary-400 hover:underline"
                                                    >
                                                        {author}
                                                    </Link>
                                                    {separator}
                                                </span>
                                            );
                                        })}
                                    </div>
                                )}
                                {hasRating && (
                                    <div className="flex items-center gap-0.5 mt-2 md:hidden">
                                        {stars.map((filled, index) => (
                                            <LuStar
                                                key={`mobile-star-${index}`}
                                                className={`w-4 h-4 ${
                                                    filled
                                                        ? 'text-yellow-400 fill-yellow-400'
                                                        : 'text-gray-300 dark:text-dark-500'
                                                }`}
                                                aria-hidden
                                            />
                                        ))}
                                    </div>
                                )}
                            </div>

                            {hasRating && (
                                <div className="hidden md:flex items-center gap-0.5 shrink-0">
                                    {stars.map((filled, index) => (
                                        <LuStar
                                            key={`desktop-star-${index}`}
                                            className={`w-5 h-5 ${
                                                filled
                                                    ? 'text-yellow-400 fill-yellow-400'
                                                    : 'text-gray-300 dark:text-dark-500'
                                            }`}
                                            aria-hidden
                                        />
                                    ))}
                                </div>
                            )}
                        </div>

                        <div className="mt-3 grid grid-cols-2 xl:grid-cols-4 gap-2">
                            <MetricCardCompact
                                icon={LuCalendarDays}
                                iconContainerClassName="bg-blue-500/20 dark:bg-linear-to-br dark:from-blue-500 dark:to-blue-600"
                                iconClassName="text-blue-600 dark:text-white"
                                label={translation.get('period')}
                                value={formatRecapDateRange(
                                    item.start_date,
                                    item.end_date,
                                )}
                            />
                            <MetricCardCompact
                                icon={LuClock3}
                                iconContainerClassName="bg-purple-500/20 dark:bg-linear-to-br dark:from-purple-500 dark:to-purple-600"
                                iconClassName="text-purple-600 dark:text-white"
                                label={translation.get('reading-time')}
                                value={
                                    <MetricCardUnitValue
                                        value={formatDurationParts(
                                            item.reading_time_sec,
                                            { includeDays: true },
                                        )}
                                        size="compact"
                                    />
                                }
                            />
                            <MetricCardCompact
                                icon={LuFileText}
                                iconContainerClassName="bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600"
                                iconClassName="text-green-600 dark:text-white"
                                label={translation.get(
                                    'pages-label',
                                    item.pages_read,
                                )}
                                value={item.pages_read}
                            />
                            <MetricCardCompact
                                icon={HiOutlineBookOpen}
                                iconContainerClassName="bg-orange-500/20 dark:bg-linear-to-br dark:from-orange-500 dark:to-orange-600"
                                iconClassName="text-orange-600 dark:text-white"
                                label={translation.get('sessions')}
                                value={item.session_count}
                            />
                        </div>

                        {hasReviewNote && (
                            <div className="mt-4">
                                <div className="bg-gray-50 dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/50 rounded-lg p-3 md:p-4">
                                    <div className="flex items-start">
                                        <LuQuote className="w-5 h-5 text-primary-400 mt-0.5 mr-2.5 shrink-0" />
                                        <p className="text-sm font-medium text-gray-700 dark:text-gray-300 leading-relaxed">
                                            {item.review_note}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
