import { useCallback, useEffect, useState } from 'react';

import { StorageManager } from '../../../shared/storage-manager';
import {
    DEFAULT_READER_FONT_SIZE,
    DEFAULT_READER_LINE_SPACING,
} from '../lib/reader-theme';

const FONT_SIZE_KEY_PREFIX = 'reader_font_size_';
const LINE_SPACING_KEY_PREFIX = 'reader_line_spacing_';

const FONT_SIZE_SCALE_MIN = 0.5;
const FONT_SIZE_SCALE_MAX = 2;
const FONT_SIZE_SCALE_STEP = 0.1;

type NumericReaderStyleConfig = {
    keyPrefix: string;
    defaultValue: number;
    min: number;
    max: number;
    step: number;
    precision: number;
};

export type ReaderStyleControl = {
    value: number;
    increase: () => void;
    decrease: () => void;
};

export type UseReaderStyleResult = {
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
};

const FONT_SIZE_CONFIG: NumericReaderStyleConfig = {
    keyPrefix: FONT_SIZE_KEY_PREFIX,
    defaultValue: DEFAULT_READER_FONT_SIZE,
    min: DEFAULT_READER_FONT_SIZE * FONT_SIZE_SCALE_MIN,
    max: DEFAULT_READER_FONT_SIZE * FONT_SIZE_SCALE_MAX,
    step: DEFAULT_READER_FONT_SIZE * FONT_SIZE_SCALE_STEP,
    precision: 2,
};

const LINE_SPACING_CONFIG: NumericReaderStyleConfig = {
    keyPrefix: LINE_SPACING_KEY_PREFIX,
    defaultValue: DEFAULT_READER_LINE_SPACING,
    min: 1,
    max: 3,
    step: 0.1,
    precision: 1,
};

function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(value, max));
}

function normalizeValue(
    value: number,
    { min, max, step, precision }: NumericReaderStyleConfig,
): number {
    const clamped = clamp(value, min, max);
    const stepsFromMin = Math.round((clamped - min) / step);
    return Number((min + stepsFromMin * step).toFixed(precision));
}

function loadValue(
    bookId: string | undefined,
    config: NumericReaderStyleConfig,
): number {
    if (!bookId) {
        return config.defaultValue;
    }

    const stored = StorageManager.getByKey<unknown>(
        `${config.keyPrefix}${bookId}`,
    );
    if (typeof stored !== 'number' || !Number.isFinite(stored)) {
        return config.defaultValue;
    }

    return normalizeValue(stored, config);
}

function stepValue(
    currentValue: number,
    delta: number,
    config: NumericReaderStyleConfig,
): number {
    const normalizedCurrent = normalizeValue(currentValue, config);
    return normalizeValue(normalizedCurrent + delta, config);
}

function useNumericReaderStyle(
    bookId: string | undefined,
    config: NumericReaderStyleConfig,
): ReaderStyleControl {
    const [value, setValue] = useState(() => loadValue(bookId, config));

    useEffect(() => {
        setValue(loadValue(bookId, config));
    }, [bookId, config]);

    const persist = useCallback(
        (nextValue: number) => {
            if (bookId) {
                StorageManager.setByKey(
                    `${config.keyPrefix}${bookId}`,
                    nextValue,
                );
            }
        },
        [bookId, config.keyPrefix],
    );

    const applyDelta = useCallback(
        (delta: number) => {
            setValue((previousValue) => {
                const nextValue = stepValue(previousValue, delta, config);
                if (nextValue !== previousValue) {
                    persist(nextValue);
                }
                return nextValue;
            });
        },
        [config, persist],
    );

    const increase = useCallback(() => {
        applyDelta(config.step);
    }, [applyDelta, config.step]);

    const decrease = useCallback(() => {
        applyDelta(-config.step);
    }, [applyDelta, config.step]);

    return { value, increase, decrease };
}

export function useReaderStyle(
    bookId: string | undefined,
): UseReaderStyleResult {
    const fontSize = useNumericReaderStyle(bookId, FONT_SIZE_CONFIG);
    const lineSpacing = useNumericReaderStyle(bookId, LINE_SPACING_CONFIG);

    return { fontSize, lineSpacing };
}
