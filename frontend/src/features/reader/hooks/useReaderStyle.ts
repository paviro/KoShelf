import { useCallback, useEffect, useMemo, useState } from 'react';

import type { LibraryReaderPresentation } from '../../../shared/contracts';
import { StorageManager } from '../../../shared/storage-manager';
import { toFiniteNonNegativeNumber } from '../lib/reader-presentation';
import {
    DEFAULT_READER_BOTTOM_MARGIN,
    DEFAULT_READER_FONT_SIZE_PT,
    DEFAULT_READER_LEFT_MARGIN,
    DEFAULT_READER_LINE_SPACING,
    DEFAULT_READER_RIGHT_MARGIN,
    DEFAULT_READER_TOP_MARGIN,
    mapKoReaderFontSizePtToCssPercent,
    mapKoReaderLineSpacingPercentToCssLineHeight,
} from '../lib/reader-theme';

const READER_STYLE_KEY_PREFIX = 'reader_style_v2_';

const KOREADER_FONT_SIZE_PT_MIN = 12;
const KOREADER_FONT_SIZE_PT_MAX = 44;
const KOREADER_FONT_SIZE_PT_STEP = 1;

export type ReaderModeValue = 'auto' | 'on' | 'off';

const READER_MODE_VALUES: readonly ReaderModeValue[] = ['auto', 'on', 'off'];

type NumericReaderStyleConfig = {
    min: number;
    max: number;
    step: number;
    precision: number;
};

export type ReaderStyleControl = {
    value: number;
    increase: () => void;
    decrease: () => void;
    stepBy: (delta: number) => void;
    reset: () => void;
    isOverridden: boolean;
};

export type ReaderModeControl = {
    value: ReaderModeValue;
    setValue: (nextValue: ReaderModeValue) => void;
    reset: () => void;
    isOverridden: boolean;
};

export type ReaderToggleControl = {
    value: boolean;
    setValue: (nextValue: boolean) => void;
    reset: () => void;
    isOverridden: boolean;
};

export type UseReaderStyleResult = {
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
    leftMargin: ReaderStyleControl;
    rightMargin: ReaderStyleControl;
    topMargin: ReaderStyleControl;
    bottomMargin: ReaderStyleControl;
    hyphenation: ReaderModeControl;
    floatingPunctuation: ReaderModeControl;
    embeddedFonts: ReaderToggleControl;
    effectivePresentation: LibraryReaderPresentation;
    resetToDefaults: () => void;
    hasOverrides: boolean;
};

type ReaderStyleState = {
    fontSize: number;
    lineSpacing: number;
    leftMargin: number;
    rightMargin: number;
    topMargin: number;
    bottomMargin: number;
    hyphenation: ReaderModeValue;
    floatingPunctuation: ReaderModeValue;
    embeddedFonts: boolean;
};

type NumericStyleKey =
    | 'fontSize'
    | 'lineSpacing'
    | 'leftMargin'
    | 'rightMargin'
    | 'topMargin'
    | 'bottomMargin';

type ModeStyleKey = 'hyphenation' | 'floatingPunctuation';

const FONT_SIZE_CONFIG: NumericReaderStyleConfig = {
    min: KOREADER_FONT_SIZE_PT_MIN,
    max: KOREADER_FONT_SIZE_PT_MAX,
    step: KOREADER_FONT_SIZE_PT_STEP,
    precision: 0,
};

const LINE_SPACING_CONFIG: NumericReaderStyleConfig = {
    min: 1,
    max: 3,
    step: 0.1,
    precision: 1,
};

const LEFT_MARGIN_CONFIG: NumericReaderStyleConfig = {
    min: 0,
    max: 96,
    step: 2,
    precision: 0,
};

const RIGHT_MARGIN_CONFIG: NumericReaderStyleConfig = {
    min: 0,
    max: 96,
    step: 2,
    precision: 0,
};

const TOP_MARGIN_CONFIG: NumericReaderStyleConfig = {
    min: 0,
    max: 96,
    step: 1,
    precision: 0,
};

const BOTTOM_MARGIN_CONFIG: NumericReaderStyleConfig = {
    min: 0,
    max: 96,
    step: 1,
    precision: 0,
};

const NUMERIC_STYLE_CONFIGS: Record<NumericStyleKey, NumericReaderStyleConfig> =
    {
        fontSize: FONT_SIZE_CONFIG,
        lineSpacing: LINE_SPACING_CONFIG,
        leftMargin: LEFT_MARGIN_CONFIG,
        rightMargin: RIGHT_MARGIN_CONFIG,
        topMargin: TOP_MARGIN_CONFIG,
        bottomMargin: BOTTOM_MARGIN_CONFIG,
    };

const ASSUMED_ROOT_FONT_SIZE_PX = 16;

function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(value, max));
}

function normalizeValue(
    value: number,
    config: NumericReaderStyleConfig,
): number {
    const clamped = clamp(value, config.min, config.max);
    const stepsFromMin = Math.round((clamped - config.min) / config.step);
    return Number(
        (config.min + stepsFromMin * config.step).toFixed(config.precision),
    );
}

function stepValue(
    currentValue: number,
    delta: number,
    config: NumericReaderStyleConfig,
): number {
    const normalizedCurrent = normalizeValue(currentValue, config);
    return normalizeValue(normalizedCurrent + delta, config);
}

function areValuesEqual(a: number, b: number, precision: number): boolean {
    const factor = 10 ** precision;
    return Math.round(a * factor) === Math.round(b * factor);
}

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function isReaderModeValue(value: unknown): value is ReaderModeValue {
    return (
        typeof value === 'string' &&
        (READER_MODE_VALUES as readonly string[]).includes(value)
    );
}

function resolveModeFromBool(
    value: boolean | null | undefined,
): ReaderModeValue {
    if (value === true) {
        return 'on';
    }

    if (value === false) {
        return 'off';
    }

    return 'auto';
}

function resolveBoolFromMode(mode: ReaderModeValue): boolean | null {
    if (mode === 'on') {
        return true;
    }

    if (mode === 'off') {
        return false;
    }

    return null;
}

function resolveHorizontalMargins(
    presentation: LibraryReaderPresentation | null | undefined,
): [number, number] {
    const margins = presentation?.h_page_margins;
    if (!Array.isArray(margins) || margins.length !== 2) {
        return [DEFAULT_READER_LEFT_MARGIN, DEFAULT_READER_RIGHT_MARGIN];
    }

    const left = toFiniteNonNegativeNumber(margins[0]);
    const right = toFiniteNonNegativeNumber(margins[1]);
    if (left === null || right === null) {
        return [DEFAULT_READER_LEFT_MARGIN, DEFAULT_READER_RIGHT_MARGIN];
    }

    return [left, right];
}

function resolveDefaultReaderStyleState(
    presentation: LibraryReaderPresentation | null | undefined,
): ReaderStyleState {
    const [leftMargin, rightMargin] = resolveHorizontalMargins(presentation);

    const fontSize =
        typeof presentation?.font_size_pt === 'number' &&
        Number.isFinite(presentation.font_size_pt) &&
        presentation.font_size_pt > 0
            ? presentation.font_size_pt
            : DEFAULT_READER_FONT_SIZE_PT;

    const lineSpacing =
        typeof presentation?.line_spacing_percent === 'number' &&
        Number.isFinite(presentation.line_spacing_percent) &&
        presentation.line_spacing_percent > 0
            ? mapKoReaderLineSpacingPercentToCssLineHeight(
                  presentation.line_spacing_percent,
              )
            : DEFAULT_READER_LINE_SPACING;

    return {
        fontSize: normalizeValue(fontSize, FONT_SIZE_CONFIG),
        lineSpacing: normalizeValue(lineSpacing, LINE_SPACING_CONFIG),
        leftMargin: normalizeValue(leftMargin, LEFT_MARGIN_CONFIG),
        rightMargin: normalizeValue(rightMargin, RIGHT_MARGIN_CONFIG),
        topMargin: normalizeValue(
            toFiniteNonNegativeNumber(presentation?.t_page_margin) ??
                DEFAULT_READER_TOP_MARGIN,
            TOP_MARGIN_CONFIG,
        ),
        bottomMargin: normalizeValue(
            toFiniteNonNegativeNumber(presentation?.b_page_margin) ??
                DEFAULT_READER_BOTTOM_MARGIN,
            BOTTOM_MARGIN_CONFIG,
        ),
        hyphenation: resolveModeFromBool(presentation?.hyphenation),
        floatingPunctuation: resolveModeFromBool(
            presentation?.floating_punctuation,
        ),
        embeddedFonts: presentation?.embedded_fonts !== false,
    };
}

function readNumericStoredValue(
    value: unknown,
    config: NumericReaderStyleConfig,
    fallback: number,
): number {
    if (typeof value !== 'number' || !Number.isFinite(value)) {
        return fallback;
    }

    return normalizeValue(value, config);
}

function loadStoredReaderStyleState(
    bookId: string | undefined,
    defaultState: ReaderStyleState,
): ReaderStyleState {
    if (!bookId) {
        return defaultState;
    }

    const stored = StorageManager.getByKey<unknown>(
        `${READER_STYLE_KEY_PREFIX}${bookId}`,
    );
    if (!isRecord(stored)) {
        return defaultState;
    }

    return {
        fontSize: readNumericStoredValue(
            stored.fontSize,
            FONT_SIZE_CONFIG,
            defaultState.fontSize,
        ),
        lineSpacing: readNumericStoredValue(
            stored.lineSpacing,
            LINE_SPACING_CONFIG,
            defaultState.lineSpacing,
        ),
        leftMargin: readNumericStoredValue(
            stored.leftMargin,
            LEFT_MARGIN_CONFIG,
            defaultState.leftMargin,
        ),
        rightMargin: readNumericStoredValue(
            stored.rightMargin,
            RIGHT_MARGIN_CONFIG,
            defaultState.rightMargin,
        ),
        topMargin: readNumericStoredValue(
            stored.topMargin,
            TOP_MARGIN_CONFIG,
            defaultState.topMargin,
        ),
        bottomMargin: readNumericStoredValue(
            stored.bottomMargin,
            BOTTOM_MARGIN_CONFIG,
            defaultState.bottomMargin,
        ),
        hyphenation: isReaderModeValue(stored.hyphenation)
            ? stored.hyphenation
            : defaultState.hyphenation,
        floatingPunctuation: isReaderModeValue(stored.floatingPunctuation)
            ? stored.floatingPunctuation
            : defaultState.floatingPunctuation,
        embeddedFonts:
            typeof stored.embeddedFonts === 'boolean'
                ? stored.embeddedFonts
                : defaultState.embeddedFonts,
    };
}

function areReaderStyleStatesEqual(
    a: ReaderStyleState,
    b: ReaderStyleState,
): boolean {
    return (
        areValuesEqual(a.fontSize, b.fontSize, FONT_SIZE_CONFIG.precision) &&
        areValuesEqual(
            a.lineSpacing,
            b.lineSpacing,
            LINE_SPACING_CONFIG.precision,
        ) &&
        areValuesEqual(
            a.leftMargin,
            b.leftMargin,
            LEFT_MARGIN_CONFIG.precision,
        ) &&
        areValuesEqual(
            a.rightMargin,
            b.rightMargin,
            RIGHT_MARGIN_CONFIG.precision,
        ) &&
        areValuesEqual(a.topMargin, b.topMargin, TOP_MARGIN_CONFIG.precision) &&
        areValuesEqual(
            a.bottomMargin,
            b.bottomMargin,
            BOTTOM_MARGIN_CONFIG.precision,
        ) &&
        a.hyphenation === b.hyphenation &&
        a.floatingPunctuation === b.floatingPunctuation &&
        a.embeddedFonts === b.embeddedFonts
    );
}

function resolveApproxLineHeightPx(
    fontSizePt: number,
    lineSpacing: number,
): number {
    const normalizedFontSizePt =
        Number.isFinite(fontSizePt) && fontSizePt > 0
            ? fontSizePt
            : DEFAULT_READER_FONT_SIZE_PT;
    const normalizedLineSpacing =
        Number.isFinite(lineSpacing) && lineSpacing > 0
            ? lineSpacing
            : DEFAULT_READER_LINE_SPACING;
    const normalizedFontSizePercent =
        mapKoReaderFontSizePtToCssPercent(normalizedFontSizePt);

    const lineHeightPx =
        ASSUMED_ROOT_FONT_SIZE_PX *
        (normalizedFontSizePercent / 100) *
        normalizedLineSpacing;

    return Math.max(1, Math.round(lineHeightPx));
}

function createNumericControl(
    value: number,
    defaultValue: number,
    precision: number,
    increaseStep: number,
    stepBy: (delta: number) => void,
    reset: () => void,
): ReaderStyleControl {
    return {
        value,
        increase: () => stepBy(increaseStep),
        decrease: () => stepBy(-increaseStep),
        stepBy,
        reset,
        isOverridden: !areValuesEqual(value, defaultValue, precision),
    };
}

export function useReaderStyle(
    bookId: string | undefined,
    presentation: LibraryReaderPresentation | null | undefined,
): UseReaderStyleResult {
    const defaultState = useMemo(
        () => resolveDefaultReaderStyleState(presentation),
        [presentation],
    );

    const [styleState, setStyleState] = useState<ReaderStyleState>(() =>
        loadStoredReaderStyleState(bookId, defaultState),
    );

    useEffect(() => {
        setStyleState(loadStoredReaderStyleState(bookId, defaultState));
    }, [bookId, defaultState]);

    useEffect(() => {
        if (!bookId) {
            return;
        }

        const storageKey = `${READER_STYLE_KEY_PREFIX}${bookId}`;
        if (areReaderStyleStatesEqual(styleState, defaultState)) {
            StorageManager.removeByKey(storageKey);
            return;
        }

        StorageManager.setByKey(storageKey, styleState);
    }, [bookId, defaultState, styleState]);

    const stepNumericStyle = useCallback(
        (key: NumericStyleKey, delta: number) => {
            const config = NUMERIC_STYLE_CONFIGS[key];
            setStyleState((previousState) => {
                const previousValue = previousState[key];
                const nextValue = stepValue(previousValue, delta, config);
                if (
                    areValuesEqual(previousValue, nextValue, config.precision)
                ) {
                    return previousState;
                }

                return {
                    ...previousState,
                    [key]: nextValue,
                };
            });
        },
        [],
    );

    const setModeStyle = useCallback(
        (key: ModeStyleKey, nextValue: ReaderModeValue) => {
            setStyleState((previousState) => {
                if (previousState[key] === nextValue) {
                    return previousState;
                }

                return {
                    ...previousState,
                    [key]: nextValue,
                };
            });
        },
        [],
    );

    const setEmbeddedFonts = useCallback((nextValue: boolean) => {
        setStyleState((previousState) => {
            if (previousState.embeddedFonts === nextValue) {
                return previousState;
            }

            return {
                ...previousState,
                embeddedFonts: nextValue,
            };
        });
    }, []);

    const resetNumericStyle = useCallback(
        (key: NumericStyleKey) => {
            const config = NUMERIC_STYLE_CONFIGS[key];
            setStyleState((previousState) => {
                const nextValue = defaultState[key];
                if (
                    areValuesEqual(
                        previousState[key],
                        nextValue,
                        config.precision,
                    )
                ) {
                    return previousState;
                }

                return {
                    ...previousState,
                    [key]: nextValue,
                };
            });
        },
        [defaultState],
    );

    const resetModeStyle = useCallback(
        (key: ModeStyleKey) => {
            setStyleState((previousState) => {
                if (previousState[key] === defaultState[key]) {
                    return previousState;
                }

                return {
                    ...previousState,
                    [key]: defaultState[key],
                };
            });
        },
        [defaultState],
    );

    const resetEmbeddedFonts = useCallback(() => {
        setStyleState((previousState) => {
            if (previousState.embeddedFonts === defaultState.embeddedFonts) {
                return previousState;
            }

            return {
                ...previousState,
                embeddedFonts: defaultState.embeddedFonts,
            };
        });
    }, [defaultState.embeddedFonts]);

    const marginLineStepPx = useMemo(
        () =>
            resolveApproxLineHeightPx(
                styleState.fontSize,
                styleState.lineSpacing,
            ),
        [styleState.fontSize, styleState.lineSpacing],
    );

    const numericControls = useMemo(() => {
        const makeNumericControl = (
            key: NumericStyleKey,
            increaseStep = NUMERIC_STYLE_CONFIGS[key].step,
        ): ReaderStyleControl =>
            createNumericControl(
                styleState[key],
                defaultState[key],
                NUMERIC_STYLE_CONFIGS[key].precision,
                increaseStep,
                (delta) => stepNumericStyle(key, delta),
                () => resetNumericStyle(key),
            );

        return {
            fontSize: makeNumericControl('fontSize'),
            lineSpacing: makeNumericControl('lineSpacing'),
            leftMargin: makeNumericControl('leftMargin'),
            rightMargin: makeNumericControl('rightMargin'),
            topMargin: makeNumericControl('topMargin', marginLineStepPx),
            bottomMargin: makeNumericControl('bottomMargin', marginLineStepPx),
        };
    }, [
        defaultState,
        marginLineStepPx,
        resetNumericStyle,
        stepNumericStyle,
        styleState,
    ]);

    const modeControls = useMemo<{
        hyphenation: ReaderModeControl;
        floatingPunctuation: ReaderModeControl;
    }>(
        () => ({
            hyphenation: {
                value: styleState.hyphenation,
                setValue: (nextValue) => setModeStyle('hyphenation', nextValue),
                reset: () => resetModeStyle('hyphenation'),
                isOverridden:
                    styleState.hyphenation !== defaultState.hyphenation,
            },
            floatingPunctuation: {
                value: styleState.floatingPunctuation,
                setValue: (nextValue) =>
                    setModeStyle('floatingPunctuation', nextValue),
                reset: () => resetModeStyle('floatingPunctuation'),
                isOverridden:
                    styleState.floatingPunctuation !==
                    defaultState.floatingPunctuation,
            },
        }),
        [
            defaultState.floatingPunctuation,
            defaultState.hyphenation,
            resetModeStyle,
            setModeStyle,
            styleState.floatingPunctuation,
            styleState.hyphenation,
        ],
    );

    const embeddedFonts = useMemo<ReaderToggleControl>(
        () => ({
            value: styleState.embeddedFonts,
            setValue: setEmbeddedFonts,
            reset: resetEmbeddedFonts,
            isOverridden:
                styleState.embeddedFonts !== defaultState.embeddedFonts,
        }),
        [
            defaultState.embeddedFonts,
            resetEmbeddedFonts,
            setEmbeddedFonts,
            styleState.embeddedFonts,
        ],
    );

    const effectivePresentation = useMemo<LibraryReaderPresentation>(
        () => ({
            ...(presentation ?? {}),
            h_page_margins: [styleState.leftMargin, styleState.rightMargin],
            t_page_margin: undefined,
            b_page_margin: undefined,
            hyphenation: resolveBoolFromMode(styleState.hyphenation),
            floating_punctuation: resolveBoolFromMode(
                styleState.floatingPunctuation,
            ),
            embedded_fonts: styleState.embeddedFonts,
        }),
        [
            presentation,
            styleState.embeddedFonts,
            styleState.floatingPunctuation,
            styleState.hyphenation,
            styleState.leftMargin,
            styleState.rightMargin,
        ],
    );

    const resetToDefaults = useCallback(() => {
        setStyleState(defaultState);
    }, [defaultState]);

    const hasOverrides = useMemo(
        () => !areReaderStyleStatesEqual(styleState, defaultState),
        [defaultState, styleState],
    );

    return {
        fontSize: numericControls.fontSize,
        lineSpacing: numericControls.lineSpacing,
        leftMargin: numericControls.leftMargin,
        rightMargin: numericControls.rightMargin,
        topMargin: numericControls.topMargin,
        bottomMargin: numericControls.bottomMargin,
        hyphenation: modeControls.hyphenation,
        floatingPunctuation: modeControls.floatingPunctuation,
        embeddedFonts,
        effectivePresentation,
        resetToDefaults,
        hasOverrides,
    };
}
