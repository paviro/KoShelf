import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import { LibraryAnnotationCard } from '../components/LibraryAnnotationCard';
import type { LibraryAnnotation } from '../api/library-data';

type LibraryHighlightsSectionProps = {
    annotations: LibraryAnnotation[];
    visible: boolean;
    onToggle: () => void;
};

export function LibraryHighlightsSection({
    annotations,
    visible,
    onToggle,
}: LibraryHighlightsSectionProps) {
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
        >
            <div className="space-y-6">
                {annotations.map((annotation, index) => (
                    <LibraryAnnotationCard
                        key={`${annotation.datetime ?? ''}-${annotation.pageno ?? ''}-${index}`}
                        annotation={annotation}
                        variant="highlight"
                    />
                ))}
            </div>
        </CollapsibleSection>
    );
}
