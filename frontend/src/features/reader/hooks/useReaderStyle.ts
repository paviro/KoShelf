import { useCallback, useEffect, useMemo, useState } from 'react';

import type { LibraryReaderPresentation } from '../../../shared/contracts';
import { StorageManager } from '../../../shared/storage-manager';
import {
    toFiniteNonNegativeNumber,
    DEFAULT_READER_BOTTOM_MARGIN,
    DEFAULT_READER_FONT_SIZE_PT,
    DEFAULT_READER_LINE_SPACING,
    DEFAULT_READER_TOP_MARGIN,
    mapKoReaderFontSizePtToCssPercent,
    mapKoReaderLineSpacingPercentToCssLineHeight,
    resolveHorizontalMarginsPx,
    resolveWordSpacingPercent,
} from '../lib/reader-theme';

const READER_STYLE_KEY_PREFIX = 'reader_style_';

const KOREADER_FONT_SIZE_PT_MIN = 12;
const KOREADER_FONT_SIZE_PT_MAX = 44;
const KOREADER_FONT_SIZE_PT_STEP = 1;

export type ReaderModeValue = 'auto' | 'on' | 'off';

type ReaderStyleBasis = 'book' | 'koshelf';

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

type UseReaderStyleResult = {
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
    wordSpacing: ReaderStyleControl;
    leftMargin: ReaderStyleControl;
    rightMargin: ReaderStyleControl;
    topMargin: ReaderStyleControl;
    bottomMargin: ReaderStyleControl;
    hyphenation: ReaderModeControl;
    floatingPunctuation: ReaderModeControl;
    embeddedFonts: ReaderToggleControl;
    effectivePresentation: LibraryReaderPresentation;
    resetToBookDefaults: () => void;
    resetToKoShelfDefaults: () => void;
    hasBookOverrides: boolean;
    hasKoShelfOverrides: boolean;
    hasDistinctBookDefaults: boolean;
};

type ReaderStyleState = {
    fontSize: number;
    lineSpacing: number;
    wordSpacing: number;
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
    | 'wordSpacing'
    | 'leftMargin'
    | 'rightMargin'
    | 'topMargin'
    | 'bottomMargin';

type ModeStyleKey = 'hyphenation' | 'floatingPunctuation';

const NUMERIC_STYLE_KEYS: readonly NumericStyleKey[] = [
    'fontSize',
    'lineSpacing',
    'wordSpacing',
    'leftMargin',
    'rightMargin',
    'topMargin',
    'bottomMargin',
];

const MODE_STYLE_KEYS: readonly ModeStyleKey[] = [
    'hyphenation',
    'floatingPunctuation',
];

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

const WORD_SPACING_CONFIG: NumericReaderStyleConfig = {
    min: 50,
    max: 200,
    step: 5,
    precision: 0,
};

const NUMERIC_STYLE_CONFIGS: Record<NumericStyleKey, NumericReaderStyleConfig> =
    {
        fontSize: FONT_SIZE_CONFIG,
        lineSpacing: LINE_SPACING_CONFIG,
        wordSpacing: WORD_SPACING_CONFIG,
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

function resolveDefaultReaderStyleState(
    presentation: LibraryReaderPresentation | null | undefined,
): ReaderStyleState {
    const [leftMargin, rightMargin] = resolveHorizontalMarginsPx(presentation);

    const fontSize =
        typeof presentation?.font_size_pt === 'number' &&
        Number.isFinite(presentation.font_size_pt) &&
        presentation.font_size_pt > 0
            ? presentation.font_size_pt
            : DEFAULT_READER_FONT_SIZE_PT;

    const lineSpacing =
        typeof presentation?.line_spacing_percentage === 'number' &&
        Number.isFinite(presentation.line_spacing_percentage) &&
        presentation.line_spacing_percentage > 0
            ? mapKoReaderLineSpacingPercentToCssLineHeight(
                  presentation.line_spacing_percentage,
              )
            : DEFAULT_READER_LINE_SPACING;

    return {
        fontSize: normalizeValue(fontSize, FONT_SIZE_CONFIG),
        lineSpacing: normalizeValue(lineSpacing, LINE_SPACING_CONFIG),
        wordSpacing: normalizeValue(
            resolveWordSpacingPercent(presentation),
            WORD_SPACING_CONFIG,
        ),
        leftMargin: normalizeValue(leftMargin, LEFT_MARGIN_CONFIG),
        rightMargin: normalizeValue(rightMargin, RIGHT_MARGIN_CONFIG),
        topMargin: normalizeValue(
            toFiniteNonNegativeNumber(presentation?.top_margin) ??
                DEFAULT_READER_TOP_MARGIN,
            TOP_MARGIN_CONFIG,
        ),
        bottomMargin: normalizeValue(
            toFiniteNonNegativeNumber(presentation?.bottom_margin) ??
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

type LoadedReaderStyle = {
    basis: ReaderStyleBasis;
    style: ReaderStyleState;
};

function loadStoredReaderStyle(
    bookId: string | undefined,
    bookDefaultState: ReaderStyleState,
    koshelfDefaultState: ReaderStyleState,
): LoadedReaderStyle {
    if (!bookId) {
        return { basis: 'koshelf', style: koshelfDefaultState };
    }

    const stored = StorageManager.getByKey<unknown>(
        `${READER_STYLE_KEY_PREFIX}${bookId}`,
    );
    if (!isRecord(stored)) {
        return { basis: 'koshelf', style: koshelfDefaultState };
    }

    const basis: ReaderStyleBasis =
        stored.basis === 'book' ? 'book' : 'koshelf';
    const defaults = basis === 'book' ? bookDefaultState : koshelfDefaultState;

    return {
        basis,
        style: {
            fontSize: readNumericStoredValue(
                stored.fontSize,
                FONT_SIZE_CONFIG,
                defaults.fontSize,
            ),
            lineSpacing: readNumericStoredValue(
                stored.lineSpacing,
                LINE_SPACING_CONFIG,
                defaults.lineSpacing,
            ),
            wordSpacing: readNumericStoredValue(
                stored.wordSpacing,
                WORD_SPACING_CONFIG,
                defaults.wordSpacing,
            ),
            leftMargin: readNumericStoredValue(
                stored.leftMargin,
                LEFT_MARGIN_CONFIG,
                defaults.leftMargin,
            ),
            rightMargin: readNumericStoredValue(
                stored.rightMargin,
                RIGHT_MARGIN_CONFIG,
                defaults.rightMargin,
            ),
            topMargin: readNumericStoredValue(
                stored.topMargin,
                TOP_MARGIN_CONFIG,
                defaults.topMargin,
            ),
            bottomMargin: readNumericStoredValue(
                stored.bottomMargin,
                BOTTOM_MARGIN_CONFIG,
                defaults.bottomMargin,
            ),
            hyphenation: isReaderModeValue(stored.hyphenation)
                ? stored.hyphenation
                : defaults.hyphenation,
            floatingPunctuation: isReaderModeValue(stored.floatingPunctuation)
                ? stored.floatingPunctuation
                : defaults.floatingPunctuation,
            embeddedFonts:
                typeof stored.embeddedFonts === 'boolean'
                    ? stored.embeddedFonts
                    : defaults.embeddedFonts,
        },
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
            a.wordSpacing,
            b.wordSpacing,
            WORD_SPACING_CONFIG.precision,
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

function buildStoredReaderStyle(
    basis: ReaderStyleBasis,
    styleState: ReaderStyleState,
    basisDefaults: ReaderStyleState,
): Record<string, unknown> {
    const stored: Record<string, unknown> = { basis };

    for (const key of NUMERIC_STYLE_KEYS) {
        const config = NUMERIC_STYLE_CONFIGS[key];
        if (
            !areValuesEqual(
                styleState[key],
                basisDefaults[key],
                config.precision,
            )
        ) {
            stored[key] = styleState[key];
        }
    }

    for (const key of MODE_STYLE_KEYS) {
        if (styleState[key] !== basisDefaults[key]) {
            stored[key] = styleState[key];
        }
    }

    if (styleState.embeddedFonts !== basisDefaults.embeddedFonts) {
        stored.embeddedFonts = styleState.embeddedFonts;
    }

    return stored;
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
    const bookDefaultState = useMemo(
        () => resolveDefaultReaderStyleState(presentation),
        [presentation],
    );

    const koshelfDefaultState = useMemo(
        () => resolveDefaultReaderStyleState(null),
        [],
    );

    const [state, setState] = useState<LoadedReaderStyle>(() =>
        loadStoredReaderStyle(bookId, bookDefaultState, koshelfDefaultState),
    );

    useEffect(() => {
        setState(
            loadStoredReaderStyle(
                bookId,
                bookDefaultState,
                koshelfDefaultState,
            ),
        );
    }, [bookId, bookDefaultState, koshelfDefaultState]);

    const { basis, style: styleState } = state;

    const basisDefaults = useMemo(
        () => (basis === 'book' ? bookDefaultState : koshelfDefaultState),
        [basis, bookDefaultState, koshelfDefaultState],
    );

    useEffect(() => {
        if (!bookId) {
            return;
        }

        const stored = buildStoredReaderStyle(basis, styleState, basisDefaults);
        StorageManager.setByKey(`${READER_STYLE_KEY_PREFIX}${bookId}`, stored);
    }, [basis, basisDefaults, bookId, styleState]);

    const stepNumericStyle = useCallback(
        (key: NumericStyleKey, delta: number) => {
            const config = NUMERIC_STYLE_CONFIGS[key];
            setState((prev) => {
                const previousValue = prev.style[key];
                const nextValue = stepValue(previousValue, delta, config);
                if (
                    areValuesEqual(previousValue, nextValue, config.precision)
                ) {
                    return prev;
                }

                return {
                    ...prev,
                    style: { ...prev.style, [key]: nextValue },
                };
            });
        },
        [],
    );

    const setModeStyle = useCallback(
        (key: ModeStyleKey, nextValue: ReaderModeValue) => {
            setState((prev) => {
                if (prev.style[key] === nextValue) {
                    return prev;
                }

                return {
                    ...prev,
                    style: { ...prev.style, [key]: nextValue },
                };
            });
        },
        [],
    );

    const setEmbeddedFonts = useCallback((nextValue: boolean) => {
        setState((prev) => {
            if (prev.style.embeddedFonts === nextValue) {
                return prev;
            }

            return {
                ...prev,
                style: { ...prev.style, embeddedFonts: nextValue },
            };
        });
    }, []);

    const resetNumericStyle = useCallback(
        (key: NumericStyleKey) => {
            const config = NUMERIC_STYLE_CONFIGS[key];
            setState((prev) => {
                const nextValue = basisDefaults[key];
                if (
                    areValuesEqual(prev.style[key], nextValue, config.precision)
                ) {
                    return prev;
                }

                return {
                    ...prev,
                    style: { ...prev.style, [key]: nextValue },
                };
            });
        },
        [basisDefaults],
    );

    const resetModeStyle = useCallback(
        (key: ModeStyleKey) => {
            setState((prev) => {
                if (prev.style[key] === basisDefaults[key]) {
                    return prev;
                }

                return {
                    ...prev,
                    style: { ...prev.style, [key]: basisDefaults[key] },
                };
            });
        },
        [basisDefaults],
    );

    const resetEmbeddedFonts = useCallback(() => {
        setState((prev) => {
            if (prev.style.embeddedFonts === basisDefaults.embeddedFonts) {
                return prev;
            }

            return {
                ...prev,
                style: {
                    ...prev.style,
                    embeddedFonts: basisDefaults.embeddedFonts,
                },
            };
        });
    }, [basisDefaults.embeddedFonts]);

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
                basisDefaults[key],
                NUMERIC_STYLE_CONFIGS[key].precision,
                increaseStep,
                (delta) => stepNumericStyle(key, delta),
                () => resetNumericStyle(key),
            );

        return {
            fontSize: makeNumericControl('fontSize'),
            lineSpacing: makeNumericControl('lineSpacing'),
            wordSpacing: makeNumericControl('wordSpacing'),
            leftMargin: makeNumericControl('leftMargin'),
            rightMargin: makeNumericControl('rightMargin'),
            topMargin: makeNumericControl('topMargin', marginLineStepPx),
            bottomMargin: makeNumericControl('bottomMargin', marginLineStepPx),
        };
    }, [
        basisDefaults,
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
                    styleState.hyphenation !== basisDefaults.hyphenation,
            },
            floatingPunctuation: {
                value: styleState.floatingPunctuation,
                setValue: (nextValue) =>
                    setModeStyle('floatingPunctuation', nextValue),
                reset: () => resetModeStyle('floatingPunctuation'),
                isOverridden:
                    styleState.floatingPunctuation !==
                    basisDefaults.floatingPunctuation,
            },
        }),
        [
            basisDefaults.floatingPunctuation,
            basisDefaults.hyphenation,
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
                styleState.embeddedFonts !== basisDefaults.embeddedFonts,
        }),
        [
            basisDefaults.embeddedFonts,
            resetEmbeddedFonts,
            setEmbeddedFonts,
            styleState.embeddedFonts,
        ],
    );

    const effectivePresentation = useMemo<LibraryReaderPresentation>(
        () => ({
            ...(presentation ?? {}),
            horizontal_margins: [styleState.leftMargin, styleState.rightMargin],
            top_margin: undefined,
            bottom_margin: undefined,
            hyphenation: resolveBoolFromMode(styleState.hyphenation),
            floating_punctuation: resolveBoolFromMode(
                styleState.floatingPunctuation,
            ),
            embedded_fonts: styleState.embeddedFonts,
            word_spacing: [
                styleState.wordSpacing,
                presentation?.word_spacing?.[1] ?? styleState.wordSpacing,
            ],
        }),
        [
            presentation,
            styleState.embeddedFonts,
            styleState.floatingPunctuation,
            styleState.hyphenation,
            styleState.leftMargin,
            styleState.rightMargin,
            styleState.wordSpacing,
        ],
    );

    const resetToBookDefaults = useCallback(() => {
        setState({ basis: 'book', style: bookDefaultState });
    }, [bookDefaultState]);

    const resetToKoShelfDefaults = useCallback(() => {
        setState({ basis: 'koshelf', style: koshelfDefaultState });
    }, [koshelfDefaultState]);

    const hasBookOverrides = useMemo(
        () => !areReaderStyleStatesEqual(styleState, bookDefaultState),
        [bookDefaultState, styleState],
    );

    const hasKoShelfOverrides = useMemo(
        () => !areReaderStyleStatesEqual(styleState, koshelfDefaultState),
        [koshelfDefaultState, styleState],
    );

    const hasDistinctBookDefaults = useMemo(
        () => !areReaderStyleStatesEqual(bookDefaultState, koshelfDefaultState),
        [bookDefaultState, koshelfDefaultState],
    );

    return {
        fontSize: numericControls.fontSize,
        lineSpacing: numericControls.lineSpacing,
        wordSpacing: numericControls.wordSpacing,
        leftMargin: numericControls.leftMargin,
        rightMargin: numericControls.rightMargin,
        topMargin: numericControls.topMargin,
        bottomMargin: numericControls.bottomMargin,
        hyphenation: modeControls.hyphenation,
        floatingPunctuation: modeControls.floatingPunctuation,
        embeddedFonts,
        effectivePresentation,
        resetToBookDefaults,
        resetToKoShelfDefaults,
        hasBookOverrides,
        hasKoShelfOverrides,
        hasDistinctBookDefaults,
    };
}
