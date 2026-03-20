import { useCallback, useEffect, useState } from 'react';

import { StorageManager } from '../../../shared/storage-manager';
import { DEFAULT_READER_FONT_SIZE } from '../lib/reader-theme';

const FONT_SIZE_KEY_PREFIX = 'reader_font_size_';
const MIN_FONT_SCALE = 0.5;
const MAX_FONT_SCALE = 2;
const MIN_FONT_SIZE = DEFAULT_READER_FONT_SIZE * MIN_FONT_SCALE;
const MAX_FONT_SIZE = DEFAULT_READER_FONT_SIZE * MAX_FONT_SCALE;
const FONT_SCALE_STEP = 0.1;

export type UseReaderFontSizeResult = {
    fontSize: number;
    increase: () => void;
    decrease: () => void;
};

function clampFontSize(size: number): number {
    return Math.max(MIN_FONT_SIZE, Math.min(size, MAX_FONT_SIZE));
}

function clampFontScale(scale: number): number {
    return Math.max(MIN_FONT_SCALE, Math.min(scale, MAX_FONT_SCALE));
}

function normalizeFontScale(scale: number): number {
    const clamped = clampFontScale(scale);
    const stepsFromMin = Math.round(
        (clamped - MIN_FONT_SCALE) / FONT_SCALE_STEP,
    );
    return Number((MIN_FONT_SCALE + stepsFromMin * FONT_SCALE_STEP).toFixed(3));
}

function fontSizeFromScale(scale: number): number {
    return Number((DEFAULT_READER_FONT_SIZE * scale).toFixed(2));
}

function stepFontSize(currentSize: number, deltaScale: number): number {
    const currentScale = normalizeFontScale(
        currentSize / DEFAULT_READER_FONT_SIZE,
    );
    const nextScale = normalizeFontScale(currentScale + deltaScale);
    return fontSizeFromScale(nextScale);
}

function normalizeFontSize(size: number): number {
    const clampedSize = clampFontSize(size);
    const scale = clampedSize / DEFAULT_READER_FONT_SIZE;
    const normalizedScale = normalizeFontScale(scale);
    return fontSizeFromScale(normalizedScale);
}

function loadFontSize(bookId: string | undefined): number {
    if (!bookId) {
        return DEFAULT_READER_FONT_SIZE;
    }

    const storedSize = StorageManager.getByKey<unknown>(
        `${FONT_SIZE_KEY_PREFIX}${bookId}`,
    );

    if (typeof storedSize !== 'number' || !Number.isFinite(storedSize)) {
        return DEFAULT_READER_FONT_SIZE;
    }

    return normalizeFontSize(storedSize);
}

export function useReaderFontSize(
    bookId: string | undefined,
): UseReaderFontSizeResult {
    const [fontSize, setFontSize] = useState(() => loadFontSize(bookId));

    useEffect(() => {
        setFontSize(loadFontSize(bookId));
    }, [bookId]);

    const persist = useCallback(
        (size: number) => {
            if (bookId) {
                StorageManager.setByKey(
                    `${FONT_SIZE_KEY_PREFIX}${bookId}`,
                    size,
                );
            }
        },
        [bookId],
    );

    const applyScaleDelta = useCallback(
        (deltaScale: number) => {
            setFontSize((prev) => {
                const next = stepFontSize(prev, deltaScale);
                if (next !== prev) {
                    persist(next);
                }
                return next;
            });
        },
        [persist],
    );

    const increase = useCallback(() => {
        applyScaleDelta(FONT_SCALE_STEP);
    }, [applyScaleDelta]);

    const decrease = useCallback(() => {
        applyScaleDelta(-FONT_SCALE_STEP);
    }, [applyScaleDelta]);

    return { fontSize, increase, decrease };
}
