import { useMemo, useRef } from 'react';
import { LuNotebookPen } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { useDarkMode } from '../../../shared/lib/dom/useDarkMode';
import { DRAWER_LIST_ITEM_CLASSNAME } from '../../../shared/ui/dropdown/dropdown-styles';
import type { LibraryAnnotation } from '../../library/api/library-data';
import { useDrawerListScroll } from '../hooks/useDrawerListScroll';
import {
    groupAnnotationsByChapter,
    normalizeReaderText,
    resolveCurrentGroupIndex,
} from '../lib/reader-drawer-utils';
import { highlightColor } from '../lib/reader-highlight-colors';
import { ReaderDrawerEmptyState } from './ReaderDrawerEmptyState';

type ReaderHighlightListProps = {
    highlights: LibraryAnnotation[];
    currentChapter: string;
    currentSectionIndex: number | null;
    onSelect: (annotation: LibraryAnnotation) => void;
};

export function ReaderHighlightList({
    highlights,
    currentChapter,
    currentSectionIndex,
    onSelect,
}: ReaderHighlightListProps) {
    const groups = useMemo(
        () => groupAnnotationsByChapter(highlights),
        [highlights],
    );
    const listRef = useRef<HTMLDivElement>(null);

    const normalizedCurrent = normalizeReaderText(currentChapter);
    const currentGroupIndex = useMemo(
        () =>
            resolveCurrentGroupIndex(
                groups,
                normalizedCurrent,
                currentSectionIndex,
            ),
        [currentSectionIndex, groups, normalizedCurrent],
    );

    useDrawerListScroll(listRef, currentGroupIndex, 'data-current-chapter');

    if (highlights.length === 0) {
        return (
            <ReaderDrawerEmptyState
                icon={LuNotebookPen}
                variant="highlights"
                title={translation.get('reader-no-highlights')}
                description={translation.get(
                    'reader-no-highlights.description',
                )}
            />
        );
    }

    const isDark = useDarkMode();

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
                            {group.annotations.map((annotation, index) => (
                                <button
                                    key={index}
                                    type="button"
                                    onClick={() => onSelect(annotation)}
                                    className={DRAWER_LIST_ITEM_CLASSNAME}
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
