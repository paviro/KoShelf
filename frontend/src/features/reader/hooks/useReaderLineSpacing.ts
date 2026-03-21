import { useCallback, useEffect, useState } from 'react';

import { StorageManager } from '../../../shared/storage-manager';
import { DEFAULT_READER_LINE_SPACING } from '../lib/reader-theme';

const LINE_SPACING_KEY_PREFIX = 'reader_line_spacing_';
const MIN_LINE_SPACING = 1;
const MAX_LINE_SPACING = 3;
const LINE_SPACING_STEP = 0.1;

export type UseReaderLineSpacingResult = {
    lineSpacing: number;
    increase: () => void;
    decrease: () => void;
};

function clampLineSpacing(value: number): number {
    return Math.max(MIN_LINE_SPACING, Math.min(value, MAX_LINE_SPACING));
}

function normalizeLineSpacing(value: number): number {
    const clamped = clampLineSpacing(value);
    const stepsFromMin = Math.round(
        (clamped - MIN_LINE_SPACING) / LINE_SPACING_STEP,
    );
    return Number(
        (MIN_LINE_SPACING + stepsFromMin * LINE_SPACING_STEP).toFixed(1),
    );
}

function stepLineSpacing(current: number, delta: number): number {
    const normalized = normalizeLineSpacing(current);
    return normalizeLineSpacing(normalized + delta);
}

function loadLineSpacing(bookId: string | undefined): number {
    if (!bookId) {
        return DEFAULT_READER_LINE_SPACING;
    }

    const stored = StorageManager.getByKey<unknown>(
        `${LINE_SPACING_KEY_PREFIX}${bookId}`,
    );

    if (typeof stored !== 'number' || !Number.isFinite(stored)) {
        return DEFAULT_READER_LINE_SPACING;
    }

    return normalizeLineSpacing(stored);
}

export function useReaderLineSpacing(
    bookId: string | undefined,
): UseReaderLineSpacingResult {
    const [lineSpacing, setLineSpacing] = useState(() =>
        loadLineSpacing(bookId),
    );

    useEffect(() => {
        setLineSpacing(loadLineSpacing(bookId));
    }, [bookId]);

    const persist = useCallback(
        (value: number) => {
            if (bookId) {
                StorageManager.setByKey(
                    `${LINE_SPACING_KEY_PREFIX}${bookId}`,
                    value,
                );
            }
        },
        [bookId],
    );

    const applyDelta = useCallback(
        (delta: number) => {
            setLineSpacing((prev) => {
                const next = stepLineSpacing(prev, delta);
                if (next !== prev) {
                    persist(next);
                }
                return next;
            });
        },
        [persist],
    );

    const increase = useCallback(() => {
        applyDelta(LINE_SPACING_STEP);
    }, [applyDelta]);

    const decrease = useCallback(() => {
        applyDelta(-LINE_SPACING_STEP);
    }, [applyDelta]);

    return { lineSpacing, increase, decrease };
}
