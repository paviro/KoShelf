import { useCallback, useRef, useState } from 'react';
import { useParams } from 'react-router';

import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageErrorState } from '../../../shared/ui/feedback/PageErrorState';
import { translation } from '../../../shared/i18n';
import type { LibraryAnnotation } from '../../library/api/library-data';
import { useLibraryDetailQuery } from '../../library/hooks/useLibraryQueries';
import { ReaderDrawerPanel } from '../components/ReaderDrawerPanel';
import { ReaderHeader } from '../components/ReaderHeader';
import { ReaderNotePopover } from '../components/ReaderNotePopover';
import { ReaderScrubber } from '../components/ReaderScrubber';
import { useReaderStyle } from '../hooks/useReaderStyle';
import { useReaderKeyboardNav } from '../hooks/useReaderKeyboardNav';
import { useReaderScrubber } from '../hooks/useReaderScrubber';
import { useReaderThemeObserver } from '../hooks/useReaderThemeObserver';
import { useReaderView } from '../hooks/useReaderView';
import { resolveAnnotationTarget } from '../lib/reader-navigation-resolution';
import type {
    FoliateView,
    ReaderLocation,
    ReaderRouteProps,
} from '../model/reader-model';

export function ReaderRoute({ collection }: ReaderRouteProps) {
    const { id } = useParams();
    const viewRef = useRef<FoliateView | null>(null);
    const [location, setLocation] = useState<ReaderLocation | null>(null);
    const [drawerOpen, setDrawerOpen] = useState(false);

    const detailQuery = useLibraryDetailQuery(collection, id);
    const readerPresentation = detailQuery.data?.item.reader_presentation;

    const {
        fontSize,
        lineSpacing,
        wordSpacing,
        leftMargin,
        rightMargin,
        topMargin,
        bottomMargin,
        hyphenation,
        floatingPunctuation,
        embeddedFonts,
        effectivePresentation,
        resetToBookDefaults,
        resetToKoShelfDefaults,
        hasBookOverrides,
        hasKoShelfOverrides,
        hasDistinctBookDefaults,
    } = useReaderStyle(id, readerPresentation);
    const scrubber = useReaderScrubber(viewRef);

    const {
        containerRef,
        loading,
        error,
        backHref,
        title,
        chapterLabel,
        chapterHref,
        currentSectionIndex,
        toc,
        highlights,
        bookmarks,
        activeNote,
        dismissNote,
        goTo,
        handleBackClick,
        handlePrev,
        handleNext,
    } = useReaderView(
        collection,
        viewRef,
        setLocation,
        scrubber.scrubSettlingRef,
        scrubber.setDragFraction,
        {
            data: detailQuery.data,
            error: detailQuery.error,
            isError: detailQuery.isError,
        },
        fontSize.value,
        lineSpacing.value,
        effectivePresentation,
    );

    useReaderKeyboardNav(handlePrev, handleNext);
    useReaderThemeObserver(
        viewRef,
        fontSize.value,
        lineSpacing.value,
        effectivePresentation,
    );

    const handleTocSelect = useCallback(
        (href: string) => {
            setDrawerOpen(false);
            goTo(href);
        },
        [goTo],
    );

    const handleAnnotationSelect = useCallback(
        async (annotation: LibraryAnnotation) => {
            setDrawerOpen(false);
            const view = viewRef.current;
            if (!view) {
                return;
            }
            const target = await resolveAnnotationTarget(view, annotation);
            if (target !== null) {
                goTo(target);
            }
        },
        [goTo, viewRef],
    );

    const displayFraction = scrubber.dragFraction ?? location?.fraction ?? 0;
    const progressPercent = Math.round(displayFraction * 100);
    const hasError = error !== null;

    return (
        <div className="fixed inset-0 z-50 flex flex-col bg-white dark:bg-dark-925">
            <ReaderHeader
                title={title}
                chapterLabel={chapterLabel}
                backHref={backHref}
                onBackClick={handleBackClick}
                fontSize={fontSize}
                lineSpacing={lineSpacing}
                wordSpacing={wordSpacing}
                leftMargin={leftMargin}
                rightMargin={rightMargin}
                topMargin={topMargin}
                bottomMargin={bottomMargin}
                hyphenation={hyphenation}
                floatingPunctuation={floatingPunctuation}
                embeddedFonts={embeddedFonts}
                onResetBookDefaults={resetToBookDefaults}
                canResetBookDefaults={hasBookOverrides}
                onResetKoShelfDefaults={resetToKoShelfDefaults}
                canResetKoShelfDefaults={hasKoShelfOverrides}
                hasDistinctBookDefaults={hasDistinctBookDefaults}
                onDrawerOpen={() => setDrawerOpen(true)}
            />

            <main className="flex-1 relative overflow-hidden bg-white dark:bg-dark-925">
                {loading && !error && (
                    <div className="absolute inset-0 flex items-center justify-center">
                        <LoadingSpinner
                            size="lg"
                            srLabel={translation.get('reader-loading')}
                        />
                    </div>
                )}

                {hasError && <PageErrorState error={error} layout="overlay" />}

                <div
                    ref={containerRef}
                    className="w-full bg-white dark:bg-dark-925"
                    style={{
                        height: `max(0px, calc(100% - ${topMargin.value}px - ${bottomMargin.value}px))`,
                        marginTop: `${topMargin.value}px`,
                        visibility: loading ? 'hidden' : 'visible',
                    }}
                />

                <ReaderNotePopover
                    open={activeNote !== null}
                    note={activeNote?.note ?? null}
                    onDismiss={dismissNote}
                />
            </main>

            {!hasError && (
                <ReaderScrubber
                    trackRef={scrubber.trackRef}
                    dragging={scrubber.dragging}
                    progressPercent={progressPercent}
                    onPrev={handlePrev}
                    onNext={handleNext}
                    onScrubStart={scrubber.handleScrubStart}
                    onScrubMove={scrubber.handleScrubMove}
                    onScrubEnd={scrubber.handleScrubEnd}
                />
            )}

            {!hasError && (
                <ReaderDrawerPanel
                    open={drawerOpen}
                    onClose={() => setDrawerOpen(false)}
                    toc={toc}
                    highlights={highlights}
                    bookmarks={bookmarks}
                    currentChapter={chapterLabel}
                    currentChapterHref={chapterHref}
                    currentSectionIndex={currentSectionIndex}
                    onTocSelect={handleTocSelect}
                    onHighlightSelect={handleAnnotationSelect}
                    onBookmarkSelect={handleAnnotationSelect}
                />
            )}
        </div>
    );
}
