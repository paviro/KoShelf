import { useState } from 'react';

import { translation } from '../../../shared/i18n';
import {
    TabbedDrawer,
    type TabbedDrawerTab,
} from '../../../shared/ui/drawer/TabbedDrawer';
import type { LibraryAnnotation } from '../../library/api/library-data';
import type { TocEntry } from '../model/reader-model';
import { ReaderBookmarkList } from './ReaderBookmarkList';
import { ReaderHighlightList } from './ReaderHighlightList';
import { ReaderTocList } from './ReaderTocList';

type DrawerTab = 'contents' | 'highlights' | 'bookmarks';

type ReaderDrawerPanelProps = {
    open: boolean;
    onClose: () => void;
    toc: TocEntry[];
    highlights: LibraryAnnotation[];
    bookmarks: LibraryAnnotation[];
    currentChapter: string;
    currentChapterHref: string | null;
    currentSectionIndex: number | null;
    onTocSelect: (href: string) => void;
    onHighlightSelect: (annotation: LibraryAnnotation) => void;
    onBookmarkSelect: (annotation: LibraryAnnotation) => void;
};

export function ReaderDrawerPanel({
    open,
    onClose,
    toc,
    highlights,
    bookmarks,
    currentChapter,
    currentChapterHref,
    currentSectionIndex,
    onTocSelect,
    onHighlightSelect,
    onBookmarkSelect,
}: ReaderDrawerPanelProps) {
    const [activeTab, setActiveTab] = useState<DrawerTab>('contents');
    const tabs: TabbedDrawerTab<DrawerTab>[] = [
        {
            id: 'contents',
            label: translation.get('reader.contents'),
            content: (
                <ReaderTocList
                    toc={toc}
                    currentChapter={currentChapter}
                    currentChapterHref={currentChapterHref}
                    currentSectionIndex={currentSectionIndex}
                    onSelect={onTocSelect}
                />
            ),
        },
        {
            id: 'highlights',
            label: translation.get('highlights'),
            content: (
                <ReaderHighlightList
                    highlights={highlights}
                    currentChapter={currentChapter}
                    currentSectionIndex={currentSectionIndex}
                    onSelect={onHighlightSelect}
                />
            ),
        },
        {
            id: 'bookmarks',
            label: translation.get('bookmarks'),
            content: (
                <ReaderBookmarkList
                    bookmarks={bookmarks}
                    currentChapter={currentChapter}
                    currentSectionIndex={currentSectionIndex}
                    onSelect={onBookmarkSelect}
                />
            ),
        },
    ];

    return (
        <TabbedDrawer
            open={open}
            onClose={onClose}
            ariaLabel={translation.get('reader-drawer.aria-label')}
            tabListAriaLabel={translation.get('reader-drawer.aria-label')}
            tabs={tabs}
            activeTab={activeTab}
            onTabChange={setActiveTab}
        />
    );
}
