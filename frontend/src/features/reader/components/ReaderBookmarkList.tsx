import { useEffect, useMemo, useRef } from 'react';
import { BsBookmarkFill } from 'react-icons/bs';

import { translation } from '../../../shared/i18n';
import type { LibraryAnnotation } from '../../library/api/library-data';
import {
    normalizeReaderText,
    resolveAnnotationSectionIndex,
    resolveSectionCandidates,
} from '../lib/reader-drawer-utils';
import { ReaderDrawerEmptyState } from './ReaderDrawerEmptyState';

type ChapterGroup = {
    chapter: string;
    bookmarks: LibraryAnnotation[];
};

type ReaderBookmarkListProps = {
    bookmarks: LibraryAnnotation[];
    currentChapter: string;
    currentSectionIndex: number | null;
    onSelect: (annotation: LibraryAnnotation) => void;
};

function groupByChapter(bookmarks: LibraryAnnotation[]): ChapterGroup[] {
    const groups: ChapterGroup[] = [];
    let current: ChapterGroup | null = null;

    for (const bookmark of bookmarks) {
        const chapter = bookmark.chapter ?? '';
        if (!current || current.chapter !== chapter) {
            current = { chapter, bookmarks: [] };
            groups.push(current);
        }
        current.bookmarks.push(bookmark);
    }

    return groups;
}
export function ReaderBookmarkList({
    bookmarks,
    currentChapter,
    currentSectionIndex,
    onSelect,
}: ReaderBookmarkListProps) {
    const groups = useMemo(() => groupByChapter(bookmarks), [bookmarks]);
    const listRef = useRef<HTMLDivElement>(null);

    const normalizedCurrent = normalizeReaderText(currentChapter);
    const currentGroupIndex = useMemo(() => {
        const matchedByChapter = groups.findIndex(
            (group) =>
                group.chapter &&
                normalizeReaderText(group.chapter) === normalizedCurrent,
        );
        if (matchedByChapter >= 0) {
            return matchedByChapter;
        }

        const candidateSectionIndexes =
            resolveSectionCandidates(currentSectionIndex);

        for (const sectionIndex of candidateSectionIndexes) {
            const matchedBySection = groups.findIndex((group) =>
                group.bookmarks.some(
                    (bookmark) =>
                        resolveAnnotationSectionIndex(bookmark) ===
                        sectionIndex,
                ),
            );
            if (matchedBySection >= 0) {
                return matchedBySection;
            }
        }

        return -1;
    }, [currentSectionIndex, groups, normalizedCurrent]);

    useEffect(() => {
        if (currentGroupIndex < 0 || !listRef.current) {
            return;
        }

        const scrollContainer = listRef.current.closest<HTMLElement>(
            '[data-tabbed-drawer-scroll-container]',
        );
        if (scrollContainer) {
            scrollContainer.style.overflowY = 'hidden';
        }

        const restoreOverflow = () => {
            if (scrollContainer) {
                scrollContainer.style.overflowY = '';
            }
        };

        const scroll = () => {
            const el = listRef.current?.querySelector<HTMLElement>(
                '[data-current-chapter]',
            );
            if (!el) {
                return;
            }

            el.scrollIntoView({
                block: 'center',
                inline: 'nearest',
            });
        };

        const frameId = requestAnimationFrame(scroll);
        const timeoutId = setTimeout(scroll, 350);
        const restoreOverflowId = window.setTimeout(restoreOverflow, 425);
        return () => {
            cancelAnimationFrame(frameId);
            clearTimeout(timeoutId);
            window.clearTimeout(restoreOverflowId);
            restoreOverflow();
        };
    }, [currentGroupIndex]);

    if (bookmarks.length === 0) {
        return (
            <ReaderDrawerEmptyState
                icon={BsBookmarkFill}
                variant="bookmarks"
                title={translation.get('reader-no-bookmarks')}
                description={translation.get('reader-no-bookmarks-description')}
            />
        );
    }

    return (
        <div ref={listRef} className="flex flex-col gap-4 py-2">
            {groups.map((group, groupIndex) => {
                const isCurrent = groupIndex === currentGroupIndex;

                return (
                    <div
                        key={groupIndex}
                        data-current-chapter={isCurrent || undefined}
                    >
                        {group.chapter && (
                            <p
                                className={`text-xs font-semibold uppercase tracking-wide px-2 pb-1.5 ${
                                    isCurrent
                                        ? 'text-amber-700 dark:text-amber-300'
                                        : 'text-gray-400 dark:text-dark-400'
                                }`}
                            >
                                {group.chapter}
                            </p>
                        )}
                        <div className="flex flex-col gap-0.5">
                            {group.bookmarks.map((annotation, index) => (
                                <button
                                    key={index}
                                    type="button"
                                    onClick={() => onSelect(annotation)}
                                    className="cursor-pointer flex items-start gap-3 text-left px-2 py-2.5 rounded-lg transition-colors duration-150 hover:bg-gray-100 dark:hover:bg-dark-700/50"
                                >
                                    <span className="w-5 h-5 rounded-md shrink-0 mt-0.5 bg-yellow-100 dark:bg-yellow-500/20 text-yellow-600 dark:text-yellow-300 flex items-center justify-center">
                                        <BsBookmarkFill
                                            className="w-2.5 h-2.5"
                                            aria-hidden="true"
                                        />
                                    </span>
                                    <div className="min-w-0 flex-1">
                                        {annotation.text ? (
                                            <p className="text-[0.925rem] text-gray-700 dark:text-dark-200 line-clamp-3 leading-snug">
                                                {annotation.text}
                                            </p>
                                        ) : (
                                            <p className="text-[0.925rem] text-gray-500 dark:text-dark-300 leading-snug">
                                                {translation.get(
                                                    'page-bookmark',
                                                )}
                                            </p>
                                        )}
                                        {typeof annotation.pageno ===
                                            'number' && (
                                            <p className="mt-0.5 text-xs text-gray-400 dark:text-dark-400">
                                                {translation.get(
                                                    'page-number',
                                                    annotation.pageno,
                                                )}
                                            </p>
                                        )}
                                    </div>
                                </button>
                            ))}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
