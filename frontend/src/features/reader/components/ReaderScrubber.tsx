import { LuChevronLeft, LuChevronRight } from 'react-icons/lu';
import type { PointerEvent as ReactPointerEvent, RefObject } from 'react';

import { translation } from '../../../shared/i18n';

type ReaderScrubberProps = {
    trackRef: RefObject<HTMLDivElement | null>;
    dragging: boolean;
    progressPercent: number;
    onPrev: () => void;
    onNext: () => void;
    onScrubStart: (e: ReactPointerEvent<HTMLDivElement>) => void;
    onScrubMove: (e: ReactPointerEvent<HTMLDivElement>) => void;
    onScrubEnd: (e: ReactPointerEvent<HTMLDivElement>) => void;
};

export function ReaderScrubber({
    trackRef,
    dragging,
    progressPercent,
    onPrev,
    onNext,
    onScrubStart,
    onScrubMove,
    onScrubEnd,
}: ReaderScrubberProps) {
    return (
        <footer className="flex items-center gap-3 h-[70px] md:h-[80px] px-4 md:px-6 border-t border-gray-200/50 dark:border-dark-700/50 bg-white/90 dark:bg-dark-950/75 backdrop-blur-xs shrink-0">
            <button
                type="button"
                onClick={onPrev}
                className="p-2 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors"
                aria-label={translation.get('reader-previous-page')}
            >
                <LuChevronLeft className="w-5 h-5 text-gray-600 dark:text-dark-300" />
            </button>

            <div className="flex-1 flex items-center gap-3">
                <div
                    ref={trackRef}
                    className="relative flex-1 h-6 flex items-center cursor-pointer touch-none"
                    onPointerDown={onScrubStart}
                    onPointerMove={onScrubMove}
                    onPointerUp={onScrubEnd}
                    onPointerCancel={onScrubEnd}
                    role="slider"
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-valuenow={progressPercent}
                    tabIndex={0}
                >
                    <div className="absolute inset-x-0 h-1.5 bg-primary-200 dark:bg-primary-900/40 rounded-full" />
                    <div
                        className={`absolute h-1.5 bg-primary-500 rounded-full ${dragging ? '' : 'transition-all duration-300'}`}
                        style={{ width: `${progressPercent}%` }}
                    />
                    <div
                        className={`absolute w-5 h-5 bg-white dark:bg-dark-100 rounded-full shadow-md ring-2 ring-primary-500 -translate-x-1/2 ${dragging ? 'scale-110' : 'transition-all duration-300 hover:scale-110'}`}
                        style={{ left: `${progressPercent}%` }}
                    />
                </div>
                <span className="text-xs text-gray-500 dark:text-dark-300 tabular-nums shrink-0 w-10 text-right">
                    {progressPercent}%
                </span>
            </div>

            <button
                type="button"
                onClick={onNext}
                className="p-2 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors"
                aria-label={translation.get('reader-next-page')}
            >
                <LuChevronRight className="w-5 h-5 text-gray-600 dark:text-dark-300" />
            </button>
        </footer>
    );
}
