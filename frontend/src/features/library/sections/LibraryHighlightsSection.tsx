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

type LibraryHighlightsSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
    sortOrder: AnnotationSortOrder;
    onToggleSort: () => void;
    readerBaseHref?: string | null;
    canWrite?: boolean;
    onSaveNote?: (annotationId: string, note: string | null) => void;
    onColorChange?: (annotationId: string, color: string) => void;
    onDrawerChange?: (annotationId: string, drawer: string) => void;
    onDelete?: (annotationId: string) => void;
    guardedAction?: (action: () => void) => void;
};

export function LibraryHighlightsSection({
    annotations,
    visible,
    onToggle,
    sortOrder,
    onToggleSort,
    readerBaseHref,
    canWrite = false,
    onSaveNote,
    onColorChange,
    onDrawerChange,
    onDelete,
    guardedAction,
}: LibraryHighlightsSectionProps) {
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
            sectionKey="highlights"
            defaultVisible
            accentClass="bg-linear-to-b from-amber-400 to-amber-600"
            title={translation.get('highlights-quotes')}
            titleBadge={
                <span className="bg-linear-to-r from-amber-500 to-amber-600 text-white text-sm px-3 py-1 rounded-full shadow-md font-medium">
                    {annotations.length}
                </span>
            }
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
            controls={controls}
        >
            <div className="space-y-6">
                {displayedEntries.map(({ annotation, originalIndex }) => (
                    <LibraryAnnotationCard
                        key={annotation.id}
                        annotation={annotation}
                        variant="highlight"
                        readerHref={annotationReaderHref(
                            readerBaseHref,
                            'highlight',
                            originalIndex,
                        )}
                        showEditingControls={canWrite && editing}
                        onSaveNote={
                            onSaveNote
                                ? (note) => onSaveNote(annotation.id, note)
                                : undefined
                        }
                        onColorChange={
                            onColorChange
                                ? (color) => onColorChange(annotation.id, color)
                                : undefined
                        }
                        onDrawerChange={
                            onDrawerChange
                                ? (drawer) =>
                                      onDrawerChange(annotation.id, drawer)
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
