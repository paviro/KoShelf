import { useRef, useState } from 'react';

import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { translation } from '../../../shared/i18n';
import { ReaderErrorState } from '../components/ReaderErrorState';
import { ReaderHeader } from '../components/ReaderHeader';
import { ReaderScrubber } from '../components/ReaderScrubber';
import { useReaderKeyboardNav } from '../hooks/useReaderKeyboardNav';
import { useReaderScrubber } from '../hooks/useReaderScrubber';
import { useReaderThemeObserver } from '../hooks/useReaderThemeObserver';
import { useReaderView } from '../hooks/useReaderView';
import type {
    FoliateView,
    ReaderLocation,
    ReaderRouteProps,
} from '../model/reader-model';

export function ReaderRoute({ collection }: ReaderRouteProps) {
    const viewRef = useRef<FoliateView | null>(null);
    const [location, setLocation] = useState<ReaderLocation | null>(null);

    const scrubber = useReaderScrubber(viewRef);

    const {
        containerRef,
        loading,
        error,
        backHref,
        title,
        chapterLabel,
        handleBackClick,
        handlePrev,
        handleNext,
    } = useReaderView(
        collection,
        viewRef,
        setLocation,
        scrubber.scrubSettlingRef,
        scrubber.setDragFraction,
    );

    useReaderKeyboardNav(handlePrev, handleNext);
    useReaderThemeObserver(viewRef);

    const displayFraction = scrubber.dragFraction ?? location?.fraction ?? 0;
    const progressPercent = Math.round(displayFraction * 100);

    return (
        <div className="fixed inset-0 z-50 flex flex-col bg-white dark:bg-dark-925">
            <ReaderHeader
                title={title}
                chapterLabel={chapterLabel}
                backHref={backHref}
                onBackClick={handleBackClick}
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

                {error && (
                    <ReaderErrorState
                        error={error}
                        backHref={backHref}
                        onBackClick={handleBackClick}
                    />
                )}

                <div
                    ref={containerRef}
                    className="w-full h-full bg-white dark:bg-dark-925"
                    style={{ visibility: loading ? 'hidden' : 'visible' }}
                />
            </main>

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
        </div>
    );
}
