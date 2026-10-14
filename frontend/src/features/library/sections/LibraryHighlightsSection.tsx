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
import { useEditSnapshot } from '../hooks/useEditSnapshot';
import { useEditToggle } from '../hooks/useEditToggle';

type LibraryHighlightsSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
    sortOrder: AnnotationSortOrder;
    onToggleSort: () => void;
    readerBaseHref?: string | null;
    canWrite?: boolean;
    saving?: boolean;
    metadataRevision?: string | null;
    onSaveNote?: (
        annotationId: string,
        note: string | null,
    ) => void | Promise<void>;
    onColorChange?: (
        annotationId: string,
        color: string,
    ) => void | Promise<void>;
    onDrawerChange?: (
        annotationId: string,
        drawer: string,
    ) => void | Promise<void>;
    onDelete?: (annotationId: string) => void | Promise<void>;
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
    saving = false,
    metadataRevision = null,
    onSaveNote,
    onColorChange,
    onDrawerChange,
    onDelete,
    guardedAction,
}: LibraryHighlightsSectionProps) {
    const { editing, toggle } = useEditToggle(guardedAction);
    const session = useEditSnapshot(
        { annotations, onSaveNote, onDelete, onColorChange, onDrawerChange },
        metadataRevision,
        editing,
    );
    const stale = session.stale || !canWrite;

    const displayedEntries = useMemo(
        () => sortedAnnotationEntries(session.value.annotations, sortOrder),
        [session.value.annotations, sortOrder],
    );

    const sortButton =
        visible && annotations.length > 0 ? (
            <AnnotationSortButton order={sortOrder} onToggle={onToggleSort} />
        ) : null;

    const editButton =
        (canWrite || editing) && visible ? (
            <EditSectionButton
                editing={editing}
                onToggle={() => {
                    if (!saving) toggle();
                }}
            />
        ) : null;

    const controls =
        sortButton || editButton ? (
            <>
                {sortButton}
                {editButton}
            </>
        ) : undefined;

    if (!editing && annotations.length === 0) return null;

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
            {editing && stale && (
                <p role="status">{translation.get('metadata-edit-stale')}</p>
            )}
            <fieldset
                disabled={saving || (editing && stale)}
                className="space-y-6"
            >
                {displayedEntries.map(({ annotation }) => (
                    <LibraryAnnotationCard
                        key={annotation.id}
                        annotation={annotation}
                        variant="highlight"
                        readerHref={annotationReaderHref(
                            readerBaseHref,
                            'highlight',
                            annotation.id,
                        )}
                        showEditingControls={editing}
                        onSaveNote={
                            session.value.onSaveNote
                                ? async (note) => {
                                      if (stale)
                                          throw new Error(
                                              'metadata revision is stale',
                                          );
                                      await session.value.onSaveNote?.(
                                          annotation.id,
                                          note,
                                      );
                                  }
                                : undefined
                        }
                        onColorChange={
                            session.value.onColorChange
                                ? async (color) => {
                                      if (stale)
                                          throw new Error(
                                              'metadata revision is stale',
                                          );
                                      await session.value.onColorChange?.(
                                          annotation.id,
                                          color,
                                      );
                                  }
                                : undefined
                        }
                        onDrawerChange={
                            session.value.onDrawerChange
                                ? async (drawer) => {
                                      if (stale)
                                          throw new Error(
                                              'metadata revision is stale',
                                          );
                                      await session.value.onDrawerChange?.(
                                          annotation.id,
                                          drawer,
                                      );
                                  }
                                : undefined
                        }
                        onDelete={
                            session.value.onDelete
                                ? async () => {
                                      if (stale)
                                          throw new Error(
                                              'metadata revision is stale',
                                          );
                                      await session.value.onDelete?.(
                                          annotation.id,
                                      );
                                  }
                                : undefined
                        }
                    />
                ))}
            </fieldset>
        </CollapsibleSection>
    );
}
