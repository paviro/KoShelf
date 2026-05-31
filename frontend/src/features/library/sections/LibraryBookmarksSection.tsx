import { useMemo } from 'react';

import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import { LibraryAnnotationCard } from '../components/LibraryAnnotationCard';
import { AnnotationSortButton } from '../components/AnnotationSortButton';
import { EditSectionButton } from '../components/EditSectionButton';
import type { LibraryAnnotation } from '../api/library-data';
import {
    sortedAnnotationEntries,
    type AnnotationSortOrder,
} from '../lib/annotation-sort';
import { annotationReaderHref } from '../lib/library-reader-links';
import { useEditToggle } from '../hooks/useEditToggle';

type LibraryBookmarksSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
    sortOrder: AnnotationSortOrder;
    onToggleSort: () => void;
    readerBaseHref?: string | null;
    canWrite?: boolean;
    onSaveNote?: (annotationId: string, note: string | null) => void;
    onDelete?: (annotationId: string) => void;
    guardedAction?: (action: () => void) => void;
};

export function LibraryBookmarksSection({
    annotations,
    visible,
    onToggle,
    sortOrder,
    onToggleSort,
    readerBaseHref,
    canWrite = false,
    onSaveNote,
    onDelete,
    guardedAction,
}: LibraryBookmarksSectionProps) {
    const { editing, toggle } = useEditToggle(guardedAction);

    const displayedEntries = useMemo(
        () => sortedAnnotationEntries(annotations, sortOrder),
        [annotations, sortOrder],
    );

    const sortButton =
        visible && annotations.length > 0 ? (
            <AnnotationSortButton order={sortOrder} onToggle={onToggleSort} />
        ) : null;

    const editButton =
        canWrite && visible ? (
            <EditSectionButton editing={editing} onToggle={toggle} />
        ) : null;

    const controls =
        sortButton || editButton ? (
            <>
                {sortButton}
                {editButton}
            </>
        ) : undefined;

    return (
        <CollapsibleSection
            sectionKey="bookmarks"
            defaultVisible
            accentClass="bg-linear-to-b from-orange-400 to-orange-600"
            title={translation.get('bookmarks')}
            titleBadge={
                <span className="bg-linear-to-r from-orange-500 to-orange-600 text-white text-sm px-3 py-1 rounded-full shadow-md font-medium">
                    {annotations.length}
                </span>
            }
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
            controls={controls}
        >
            <div className="space-y-3">
                {displayedEntries.map(({ annotation, originalIndex }) => (
                    <LibraryAnnotationCard
                        key={annotation.id}
                        annotation={annotation}
                        variant="bookmark"
                        readerHref={annotationReaderHref(
                            readerBaseHref,
                            'bookmark',
                            originalIndex,
                        )}
                        showEditingControls={canWrite && editing}
                        onSaveNote={
                            onSaveNote
                                ? (note) => onSaveNote(annotation.id, note)
                                : undefined
                        }
                        onDelete={
                            onDelete ? () => onDelete(annotation.id) : undefined
                        }
                    />
                ))}
            </div>
        </CollapsibleSection>
    );
}
