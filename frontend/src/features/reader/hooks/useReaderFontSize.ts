import { useCallback, useEffect, useState } from 'react';

import { StorageManager } from '../../../shared/storage-manager';
import { DEFAULT_READER_FONT_SIZE } from '../lib/reader-theme';

const FONT_SIZE_KEY_PREFIX = 'reader_font_size_';
const MIN_FONT_SIZE = 75;
const MAX_FONT_SIZE = 200;
const FONT_SIZE_STEP = 12.5;

export type UseReaderFontSizeResult = {
    fontSize: number;
    increase: () => void;
    decrease: () => void;
};

function clampFontSize(size: number): number {
    return Math.max(MIN_FONT_SIZE, Math.min(size, MAX_FONT_SIZE));
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

    return clampFontSize(storedSize);
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

    const increase = useCallback(() => {
        setFontSize((prev) => {
            const next = clampFontSize(prev + FONT_SIZE_STEP);
            if (next !== prev) persist(next);
            return next;
        });
    }, [persist]);

    const decrease = useCallback(() => {
        setFontSize((prev) => {
            const next = clampFontSize(prev - FONT_SIZE_STEP);
            if (next !== prev) persist(next);
            return next;
        });
    }, [persist]);

    return { fontSize, increase, decrease };
}
