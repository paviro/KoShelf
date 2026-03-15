import { useMemo, useState } from 'react';
import { Link } from 'react-router';
import { BsHighlighter } from 'react-icons/bs';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import {
    LuArrowUpRight,
    LuCheck,
    LuFileText,
    LuLanguages,
    LuNotebookPen,
    LuTags,
} from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import type {
    LibraryCompletions,
    LibraryDetailItem,
    LibraryItemStats,
} from '../api/library-data';
import {
    formatIsoDate,
    formatSeriesDisplay,
    formatLanguageDisplayName,
    sanitizeRichTextHtml,
    toProgressPercentage,
} from '../lib/library-detail-formatters';

type LibraryOverviewSectionProps = {
    item: LibraryDetailItem;
    itemStats: LibraryItemStats | null;
    completions: LibraryCompletions | null;
    highlightCount: number;
    noteCount: number;
    visible: boolean;
    onToggle: () => void;
};

function cardValueOrUnknown(value: number | null | undefined): string {
    if (typeof value !== 'number' || !Number.isFinite(value)) {
        return '?';
    }

    return formatNumber(value);
}

export function LibraryOverviewSection({
    item,
    itemStats,
    completions,
    highlightCount,
    noteCount,
    visible,
    onToggle,
}: LibraryOverviewSectionProps) {
    const [coverFailed, setCoverFailed] = useState(false);

    const progressPercentage = toProgressPercentage(item.progress_percentage);
    const languageDisplay = formatLanguageDisplayName(item.language);
    const showProgressStatus =
        item.status === 'reading' || item.status === 'abandoned';
    const showCompletedStatus = item.status === 'complete';
    const isBook = item.content_type === 'book';
    const seriesDisplay = formatSeriesDisplay(item.series);
    const hasSeries = Boolean(seriesDisplay);
    const hasSubjects = item.subjects.length > 0;
    const pagesCount = item.pages ?? itemStats?.pages;
    const seriesSearchBasePath = item.search_base_path?.trim() || '/';
    const seriesSearchTerm = (item.series?.name ?? '').trim();
    const sanitizedDescription = useMemo(
        () => sanitizeRichTextHtml(item.description ?? ''),
        [item.description],
    );

    return (
        <CollapsibleSection
            sectionKey="book-overview"
            defaultVisible
            accentClass="bg-linear-to-b from-primary-400 to-primary-600"
            title={
                item.content_type === 'comic'
                    ? translation.get('comic-overview')
                    : translation.get('book-overview')
            }
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
        >
            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 md:gap-8">
                <div className="lg:col-span-1 space-y-4 md:space-y-6 mb-4 md:mb-0">
                    <div className="space-y-4">
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-xl overflow-hidden shadow-lg dark:shadow-none mx-auto max-w-[280px] md:max-w-xs mb-4 relative">
                            {!coverFailed ? (
                                <img
                                    src={item.cover_url}
                                    alt={item.title}
                                    className="w-full h-auto"
                                    onError={() => setCoverFailed(true)}
                                />
                            ) : (
                                <div className="aspect-2/3 w-full flex items-center justify-center text-5xl md:text-6xl text-gray-400 dark:text-dark-400">
                                    <span aria-hidden="true">📖</span>
                                </div>
                            )}

                            {showProgressStatus && (
                                <div
                                    className="book-progress-bar progress-reading"
                                    style={{ width: `${progressPercentage}%` }}
                                />
                            )}
                        </div>

                        {showProgressStatus && (
                            <div className="@container bg-primary-50 dark:bg-dark-850/50 border border-primary-200 dark:border-dark-700/70 rounded-lg p-4 mx-auto max-w-[280px] md:max-w-xs">
                                <div className="flex flex-col @[180px]:flex-row items-center justify-center gap-3">
                                    <div className="w-10 h-10 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-lg flex items-center justify-center shrink-0">
                                        <HiOutlineBookOpen
                                            className="w-5 h-5 text-primary-600 dark:text-white"
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <div className="text-center @[180px]:text-left">
                                        <div className="text-lg font-bold text-gray-900 dark:text-white">
                                            {progressPercentage}%
                                        </div>
                                        <div className="text-sm text-gray-500 dark:text-dark-400">
                                            {translation.get(
                                                'reading-progress',
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {showCompletedStatus && (
                            <div className="@container bg-green-50 dark:bg-dark-850/50 border border-green-200 dark:border-dark-700/70 rounded-lg p-4 mx-auto max-w-[280px] md:max-w-xs">
                                <div className="flex flex-col @[180px]:flex-row items-center justify-center gap-3">
                                    <div className="w-10 h-10 bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600 rounded-lg flex items-center justify-center shrink-0">
                                        <LuCheck
                                            className="w-5 h-5 text-green-600 dark:text-white"
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <div className="text-center @[180px]:text-left">
                                        <div className="text-lg font-bold text-gray-900 dark:text-white">
                                            {translation.get(
                                                'status.completed',
                                            )}
                                        </div>
                                        <div className="text-sm text-gray-500 dark:text-dark-400">
                                            {completions?.last_completion_date
                                                ? `${translation.get('last')}: ${formatIsoDate(completions.last_completion_date)}`
                                                : translation.get(
                                                      'reading-progress',
                                                  )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                <div className="lg:col-span-3 space-y-6">
                    <div className="flex flex-wrap gap-3 sm:gap-4">
                        <div className="@container flex-1 min-w-[120px] sm:min-w-[140px] bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
                            <div className="flex flex-col @[140px]:flex-row items-center @[140px]:items-center gap-2 @[140px]:gap-3 h-full">
                                <div className="w-10 h-10 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-lg flex items-center justify-center shrink-0">
                                    <LuFileText
                                        className="w-5 h-5 text-primary-600 dark:text-white"
                                        aria-hidden="true"
                                    />
                                </div>
                                <div className="text-center @[140px]:text-left">
                                    <div className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                                        {cardValueOrUnknown(pagesCount)}
                                    </div>
                                    <div className="text-sm text-gray-500 dark:text-dark-400">
                                        {translation.get(
                                            'pages-label',
                                            pagesCount ?? 0,
                                        )}
                                    </div>
                                </div>
                            </div>
                        </div>

                        {isBook && (
                            <div className="@container flex-1 min-w-[120px] sm:min-w-[140px] bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
                                <div className="flex flex-col @[140px]:flex-row items-center @[140px]:items-center gap-2 @[140px]:gap-3 h-full">
                                    <div className="w-10 h-10 bg-amber-500/20 dark:bg-linear-to-br dark:from-amber-500 dark:to-amber-600 rounded-lg flex items-center justify-center shrink-0">
                                        <BsHighlighter
                                            className="w-5 h-5 text-amber-600 dark:text-white"
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <div className="text-center @[140px]:text-left">
                                        <div className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                                            {formatNumber(highlightCount)}
                                        </div>
                                        <div className="text-sm text-gray-500 dark:text-dark-400">
                                            {translation.get(
                                                'highlights-label',
                                                highlightCount,
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {isBook && (
                            <div className="@container flex-1 min-w-[120px] sm:min-w-[140px] bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
                                <div className="flex flex-col @[140px]:flex-row items-center @[140px]:items-center gap-2 @[140px]:gap-3 h-full">
                                    <div className="w-10 h-10 bg-indigo-500/20 dark:bg-linear-to-br dark:from-indigo-500 dark:to-indigo-600 rounded-lg flex items-center justify-center shrink-0">
                                        <LuNotebookPen
                                            className="w-5 h-5 text-indigo-600 dark:text-white"
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <div className="text-center @[140px]:text-left">
                                        <div className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                                            {formatNumber(noteCount)}
                                        </div>
                                        <div className="text-sm text-gray-500 dark:text-dark-400">
                                            {translation.get(
                                                'notes-label',
                                                noteCount,
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        <div className="@container flex-1 min-w-[120px] sm:min-w-[140px] bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
                            <div className="flex flex-col @[140px]:flex-row items-center @[140px]:items-center gap-2 @[140px]:gap-3 h-full">
                                <div className="w-10 h-10 bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600 rounded-lg flex items-center justify-center shrink-0">
                                    <LuLanguages
                                        className="w-5 h-5 text-green-600 dark:text-white"
                                        aria-hidden="true"
                                    />
                                </div>
                                <div className="text-center @[140px]:text-left">
                                    <div className="text-base md:text-lg font-bold text-gray-900 dark:text-white">
                                        {languageDisplay}
                                    </div>
                                    <div className="text-sm text-gray-500 dark:text-dark-400">
                                        {translation.get('language')}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {sanitizedDescription && (
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                                <div className="w-8 h-8 bg-purple-500/20 dark:bg-linear-to-br dark:from-purple-500 dark:to-purple-600 rounded-lg flex items-center justify-center mr-3">
                                    <LuFileText
                                        className="w-4 h-4 text-purple-600 dark:text-white"
                                        aria-hidden="true"
                                    />
                                </div>
                                {translation.get('description')}
                            </h3>
                            <div
                                className="leading-relaxed prose max-w-none book-description"
                                dangerouslySetInnerHTML={{
                                    __html: sanitizedDescription,
                                }}
                            />
                        </div>
                    )}

                    {hasSeries && (
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                            <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                                <div className="w-8 h-8 bg-pink-500/20 dark:bg-linear-to-br dark:from-pink-500 dark:to-pink-600 rounded-lg flex items-center justify-center mr-3">
                                    <HiOutlineBookOpen
                                        className="w-4 h-4 text-pink-600 dark:text-white"
                                        aria-hidden="true"
                                    />
                                </div>
                                {translation.get('series')}
                            </h4>

                            <Link
                                to={`${seriesSearchBasePath}?search=${encodeURIComponent(seriesSearchTerm)}`}
                                className="inline-flex items-center px-4 py-2 rounded-lg text-sm font-medium bg-gray-100 dark:bg-dark-700 text-primary-600 dark:text-primary-300 border border-gray-300 dark:border-dark-600 hover:bg-primary-50 dark:hover:bg-dark-650 hover:border-primary-500 hover:text-primary-700 dark:hover:text-primary-200 transition-colors"
                            >
                                {seriesDisplay}
                                <LuArrowUpRight
                                    className="w-4 h-4 ml-2"
                                    aria-hidden="true"
                                />
                            </Link>
                        </div>
                    )}

                    {hasSubjects && (
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                            <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                                <div className="w-8 h-8 bg-cyan-500/20 dark:bg-linear-to-br dark:from-cyan-500 dark:to-cyan-600 rounded-lg flex items-center justify-center mr-3">
                                    <LuTags
                                        className="w-4 h-4 text-cyan-600 dark:text-white"
                                        aria-hidden="true"
                                    />
                                </div>
                                {translation.get('genres')}
                            </h4>
                            <div className="flex flex-wrap gap-3">
                                {item.subjects.map((subject) => (
                                    <span
                                        key={subject}
                                        className="inline-flex items-center px-4 py-2 rounded-full text-sm font-medium bg-primary-100 dark:bg-primary-600/20 text-primary-700 dark:text-primary-300 border border-primary-200 dark:border-primary-600 border-opacity-30 hover:bg-primary-200 dark:hover:bg-primary-600/30 transition-colors"
                                    >
                                        {subject}
                                    </span>
                                ))}
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </CollapsibleSection>
    );
}
