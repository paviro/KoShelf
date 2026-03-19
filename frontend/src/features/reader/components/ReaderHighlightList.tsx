import { useEffect, useMemo, useRef } from 'react';
import { LuNotebookPen } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import type { LibraryAnnotation } from '../../library/api/library-data';
import {
    normalizeReaderText,
    resolveAnnotationSectionIndex,
    resolveSectionCandidates,
} from '../lib/reader-drawer-utils';
import { highlightColor } from '../lib/reader-highlight-colors';
import { ReaderDrawerEmptyState } from './ReaderDrawerEmptyState';

type ChapterGroup = {
    chapter: string;
    highlights: LibraryAnnotation[];
};

type ReaderHighlightListProps = {
    highlights: LibraryAnnotation[];
    currentChapter: string;
    currentSectionIndex: number | null;
    onSelect: (annotation: LibraryAnnotation) => void;
};

function groupByChapter(highlights: LibraryAnnotation[]): ChapterGroup[] {
    const groups: ChapterGroup[] = [];
    let current: ChapterGroup | null = null;

    for (const h of highlights) {
        const chapter = h.chapter ?? '';
        if (!current || current.chapter !== chapter) {
            current = { chapter, highlights: [] };
            groups.push(current);
        }
        current.highlights.push(h);
    }

    return groups;
}
export function ReaderHighlightList({
    highlights,
    currentChapter,
    currentSectionIndex,
    onSelect,
}: ReaderHighlightListProps) {
    const groups = useMemo(() => groupByChapter(highlights), [highlights]);
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
                group.highlights.some(
                    (highlight) =>
                        resolveAnnotationSectionIndex(highlight) ===
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

        // Use rAF to wait for paint after mount, plus a fallback timeout
        // for when the drawer is still animating on first open.
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

    if (highlights.length === 0) {
        return (
            <ReaderDrawerEmptyState
                icon={LuNotebookPen}
                variant="highlights"
                title={translation.get('reader-no-highlights')}
                description={translation.get(
                    'reader-no-highlights-description',
                )}
            />
        );
    }

    const isDark = document.documentElement.classList.contains('dark');

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
                            {group.highlights.map((annotation, index) => (
                                <button
                                    key={index}
                                    type="button"
                                    onClick={() => onSelect(annotation)}
                                    className="cursor-pointer flex items-start gap-3 text-left px-2 py-2.5 rounded-lg transition-colors duration-150 hover:bg-gray-100 dark:hover:bg-dark-700/50"
                                >
                                    <span
                                        className="w-2.5 h-2.5 rounded-full shrink-0 mt-1"
                                        style={{
                                            backgroundColor: highlightColor(
                                                annotation.color,
                                                isDark,
                                            ),
                                        }}
                                    />
                                    <div className="min-w-0 flex-1">
                                        {annotation.text && (
                                            <p className="text-[0.925rem] text-gray-700 dark:text-dark-200 line-clamp-3 leading-snug">
                                                {annotation.text}
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
