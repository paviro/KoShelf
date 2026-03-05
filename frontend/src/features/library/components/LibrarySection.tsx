import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import type { LibraryListItem } from '../api/library-data';
import type {
    LibraryCollection,
    LibrarySectionKey,
} from '../model/library-model';
import { LibraryCard } from './LibraryCard';

type LibrarySectionProps = {
    sectionKey: LibrarySectionKey;
    title: string;
    items: LibraryListItem[];
    collection: LibraryCollection;
    visible: boolean;
    onToggle: () => void;
};

const SECTION_STYLES: Record<
    LibrarySectionKey,
    { accentClass: string; badgeClass: string }
> = {
    reading: {
        accentClass: 'from-primary-400 to-primary-600',
        badgeClass: 'from-primary-500 to-primary-600',
    },
    abandoned: {
        accentClass: 'from-purple-400 to-purple-600',
        badgeClass: 'from-purple-500 to-purple-600',
    },
    completed: {
        accentClass: 'from-green-400 to-green-600',
        badgeClass: 'from-green-500 to-green-600',
    },
    unread: {
        accentClass: 'from-orange-400 to-orange-600',
        badgeClass: 'from-orange-500 to-orange-600',
    },
};

const DEFAULT_VISIBILITY: Record<LibrarySectionKey, boolean> = {
    reading: true,
    abandoned: false,
    completed: true,
    unread: true,
};

export function LibrarySection({
    sectionKey,
    title,
    items,
    collection,
    visible,
    onToggle,
}: LibrarySectionProps) {
    const style = SECTION_STYLES[sectionKey];

    return (
        <CollapsibleSection
            sectionKey={sectionKey}
            defaultVisible={DEFAULT_VISIBILITY[sectionKey]}
            accentClass={`bg-gradient-to-b ${style.accentClass}`}
            title={title}
            titleBadge={
                <span
                    className={`bg-gradient-to-r ${style.badgeClass} text-white text-sm px-3 py-1 rounded-full shadow-md font-medium`}
                >
                    {items.length}
                </span>
            }
            visible={visible}
            onToggle={onToggle}
        >
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-4 md:gap-6 mb-6 md:mb-8">
                {items.map((item) => (
                    <LibraryCard
                        key={item.id}
                        item={item}
                        collection={collection}
                        sectionKey={sectionKey}
                    />
                ))}
            </div>
        </CollapsibleSection>
    );
}
