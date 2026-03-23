import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import { LibraryAnnotationCard } from '../components/LibraryAnnotationCard';
import { EditSectionButton } from '../components/EditSectionButton';
import type { LibraryAnnotation } from '../api/library-data';
import { annotationReaderHref } from '../lib/library-reader-links';
import { useEditToggle } from '../hooks/useEditToggle';

type LibraryHighlightsSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
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
    readerBaseHref,
    canWrite = false,
    onSaveNote,
    onColorChange,
    onDrawerChange,
    onDelete,
    guardedAction,
}: LibraryHighlightsSectionProps) {
    const { editing, toggle } = useEditToggle(guardedAction);

    return (
        <CollapsibleSection
            sectionKey="highlights"
            defaultVisible
            accentClass="bg-linear-to-b from-amber-400 to-amber-600"
            title={translation.get('highlights-quotes')}
            titleBadge={
                <span className="ml-3 text-sm font-normal text-gray-500 dark:text-dark-400">
                    ({annotations.length})
                </span>
            }
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
            controls={
                canWrite && visible ? (
                    <EditSectionButton editing={editing} onToggle={toggle} />
                ) : undefined
            }
        >
            <div className="space-y-6">
                {annotations.map((annotation, index) => (
                    <LibraryAnnotationCard
                        key={annotation.id}
                        annotation={annotation}
                        variant="highlight"
                        readerHref={annotationReaderHref(
                            readerBaseHref,
                            'highlight',
                            index,
                        )}
                        canWrite={canWrite && editing}
                        onSaveNote={
                            onSaveNote
                                ? (note) =>
                                      onSaveNote(annotation.id, note)
                                : undefined
                        }
                        onColorChange={
                            onColorChange
                                ? (color) =>
                                      onColorChange(annotation.id, color)
                                : undefined
                        }
                        onDrawerChange={
                            onDrawerChange
                                ? (drawer) =>
                                      onDrawerChange(annotation.id, drawer)
                                : undefined
                        }
                        onDelete={
                            onDelete
                                ? () => onDelete(annotation.id)
                                : undefined
                        }
                    />
                ))}
            </div>
        </CollapsibleSection>
    );
}
