import { useMemo, useState } from 'react';
import { Link } from 'react-router';
import { BsHighlighter } from 'react-icons/bs';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import {
    LuArrowUpRight,
    LuBan,
    LuCheck,
    LuFileText,
    LuLanguages,
    LuNotebookPen,
    LuPencil,
    LuTags,
} from 'react-icons/lu';
import type { IconType } from 'react-icons';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';
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

const ITEM_STATUS_OPTIONS: ReadonlyArray<{
    value: string;
    labelKey: string;
    active: string;
    iconContainer: string;
    iconClassName: string;
    icon: IconType;
}> = [
    {
        value: 'reading',
        labelKey: 'status.reading-short',
        active: 'bg-primary-50 dark:bg-primary-500/10 border-primary-300 dark:border-primary-500/30 text-primary-700 dark:text-primary-300',
        iconContainer:
            'bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600',
        iconClassName: 'text-primary-600 dark:text-white',
        icon: HiOutlineBookOpen,
    },
    {
        value: 'complete',
        labelKey: 'status.completed-short',
        active: 'bg-green-50 dark:bg-green-500/10 border-green-300 dark:border-green-500/30 text-green-700 dark:text-green-300',
        iconContainer:
            'bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600',
        iconClassName: 'text-green-600 dark:text-white',
        icon: LuCheck,
    },
    {
        value: 'abandoned',
        labelKey: 'status.abandoned',
        active: 'bg-red-50 dark:bg-red-500/10 border-red-300 dark:border-red-500/30 text-red-700 dark:text-red-300',
        iconContainer:
            'bg-red-500/20 dark:bg-linear-to-br dark:from-red-500 dark:to-red-600',
        iconClassName: 'text-red-600 dark:text-white',
        icon: LuBan,
    },
];

type LibraryOverviewSectionProps = {
    item: LibraryDetailItem;
    itemStats: LibraryItemStats | null;
    completions: LibraryCompletions | null;
    highlightCount: number;
    noteCount: number;
    visible: boolean;
    onToggle: () => void;
    canWrite?: boolean;
    onStatusChange?: (status: string) => void;
    guardedAction?: (action: () => void) => void;
};

function StatusCard({
    bgClass,
    borderClass,
    iconContainerClass,
    icon: Icon,
    iconClass,
    title,
    subtitle,
}: {
    bgClass: string;
    borderClass: string;
    iconContainerClass: string;
    icon: IconType;
    iconClass: string;
    title: string;
    subtitle?: string;
}) {
    return (
        <div
            className={`@container ${bgClass} dark:bg-dark-850/50 border ${borderClass} dark:border-dark-700/70 rounded-lg p-4 mx-auto max-w-[280px] md:max-w-xs`}
        >
            <div className="flex flex-col @[180px]:flex-row items-center justify-center gap-3">
                <div
                    className={`w-10 h-10 ${iconContainerClass} rounded-lg flex items-center justify-center shrink-0`}
                >
                    <Icon
                        className={`w-5 h-5 ${iconClass}`}
                        aria-hidden="true"
                    />
                </div>
                <div className="text-center @[180px]:text-left">
                    <div className="text-lg font-bold text-gray-900 dark:text-white">
                        {title}
                    </div>
                    {subtitle && (
                        <div className="text-sm text-gray-500 dark:text-dark-400">
                            {subtitle}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

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
    canWrite = false,
    onStatusChange,
    guardedAction,
}: LibraryOverviewSectionProps) {
    const [coverFailed, setCoverFailed] = useState(false);
    const [statusModalOpen, setStatusModalOpen] = useState(false);

    const handleOpenStatusModal = () => {
        if (guardedAction) {
            guardedAction(() => setStatusModalOpen(true));
        } else {
            setStatusModalOpen(true);
        }
    };

    const handleStatusSelect = (newStatus: string) => {
        onStatusChange?.(newStatus);
        setStatusModalOpen(false);
    };

    const progressPercentage = toProgressPercentage(item.progress_percentage);
    const languageDisplay = formatLanguageDisplayName(item.language);
    const showProgressStatus = item.status === 'reading';
    const showAbandonedStatus = item.status === 'abandoned';
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
        <>
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
                controls={
                    canWrite && visible ? (
                        <button
                            type="button"
                            onClick={handleOpenStatusModal}
                            className="flex items-center justify-center w-10 h-10 rounded-lg border transition-colors backdrop-blur-xs bg-gray-100/50 dark:bg-dark-800/50 border-gray-300/50 dark:border-dark-700/50 text-gray-600 dark:text-dark-300 hover:bg-gray-200/50 dark:hover:bg-dark-700/50"
                            aria-label={translation.get('edit.aria-label')}
                        >
                            <LuPencil className="w-4 h-4" aria-hidden="true" />
                        </button>
                    ) : undefined
                }
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
                                        style={{
                                            width: `${progressPercentage}%`,
                                        }}
                                    />
                                )}
                            </div>

                            {showProgressStatus && (
                                <StatusCard
                                    bgClass="bg-primary-50"
                                    borderClass="border-primary-200"
                                    iconContainerClass="bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600"
                                    icon={HiOutlineBookOpen}
                                    iconClass="text-primary-600 dark:text-white"
                                    title={`${progressPercentage}%`}
                                    subtitle={translation.get(
                                        'reading-progress',
                                    )}
                                />
                            )}

                            {showCompletedStatus && (
                                <StatusCard
                                    bgClass="bg-green-50"
                                    borderClass="border-green-200"
                                    iconContainerClass="bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600"
                                    icon={LuCheck}
                                    iconClass="text-green-600 dark:text-white"
                                    title={translation.get('status.completed')}
                                    subtitle={
                                        completions?.last_completion_date
                                            ? `${translation.get('last')}: ${formatIsoDate(completions.last_completion_date)}`
                                            : undefined
                                    }
                                />
                            )}

                            {showAbandonedStatus && (
                                <StatusCard
                                    bgClass="bg-red-50"
                                    borderClass="border-red-200"
                                    iconContainerClass="bg-red-500/20 dark:bg-linear-to-br dark:from-red-500 dark:to-red-600"
                                    icon={LuBan}
                                    iconClass="text-red-600 dark:text-white"
                                    title={translation.get('status.abandoned')}
                                />
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

            <ModalShell
                open={statusModalOpen}
                onClose={() => setStatusModalOpen(false)}
                containerClassName="z-[60]"
                cardClassName="max-w-sm bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            >
                <div className="p-6">
                    <div className="flex items-center gap-3 mb-5">
                        <div className="w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-500/20 flex items-center justify-center shrink-0">
                            <LuPencil className="w-5 h-5 text-primary-600 dark:text-primary-400" />
                        </div>
                        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                            {translation.get('change-status')}
                        </h2>
                    </div>
                    <div className="flex flex-col gap-2">
                        {ITEM_STATUS_OPTIONS.map((opt) => {
                            const isActive = item.status === opt.value;
                            return (
                                <button
                                    key={opt.value}
                                    type="button"
                                    onClick={() =>
                                        handleStatusSelect(opt.value)
                                    }
                                    className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium border transition-all duration-200 ${
                                        isActive
                                            ? opt.active
                                            : 'bg-gray-50 dark:bg-dark-800 border-gray-200 dark:border-dark-700 text-gray-700 dark:text-dark-300 hover:bg-gray-100 dark:hover:bg-dark-700'
                                    }`}
                                >
                                    <div
                                        className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 ${opt.iconContainer}`}
                                    >
                                        <opt.icon
                                            className={`w-5 h-5 ${opt.iconClassName}`}
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <span className="flex-1 text-left">
                                        {translation.get(opt.labelKey)}
                                    </span>
                                    {isActive && (
                                        <LuCheck
                                            className="w-4 h-4 shrink-0"
                                            aria-hidden="true"
                                        />
                                    )}
                                </button>
                            );
                        })}
                    </div>
                </div>
            </ModalShell>
        </>
    );
}
