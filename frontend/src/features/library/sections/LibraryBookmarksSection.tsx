import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import { LibraryAnnotationCard } from '../components/LibraryAnnotationCard';
import type { LibraryAnnotation } from '../api/library-data';
import { annotationReaderHref } from '../lib/library-reader-links';

type LibraryBookmarksSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
    readerBaseHref?: string | null;
};

export function LibraryBookmarksSection({
    annotations,
    visible,
    onToggle,
    readerBaseHref,
}: LibraryBookmarksSectionProps) {
    return (
        <CollapsibleSection
            sectionKey="bookmarks"
            defaultVisible
            accentClass="bg-linear-to-b from-yellow-400 to-yellow-600"
            title={translation.get('bookmarks')}
            titleBadge={
                <span className="ml-3 text-sm font-normal text-gray-500 dark:text-dark-400">
                    ({annotations.length})
                </span>
            }
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
        >
            <div className="space-y-6">
                {annotations.map((annotation, index) => (
                    <LibraryAnnotationCard
                        key={`${annotation.datetime ?? ''}-${annotation.pageno ?? ''}-${index}`}
                        annotation={annotation}
                        variant="bookmark"
                        readerHref={annotationReaderHref(
                            readerBaseHref,
                            'bookmark',
                            index,
                        )}
                    />
                ))}
            </div>
        </CollapsibleSection>
    );
}
