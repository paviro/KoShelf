import {
    LuContrast,
    LuHighlighter,
    LuStrikethrough,
    LuUnderline,
} from 'react-icons/lu';
import type { IconType } from 'react-icons';

import {
    DEFAULT_HIGHLIGHT_COLOR,
    HIGHLIGHT_COLOR_NAMES,
    type HighlightColorName,
} from '../../../shared/lib/highlight-palette';

// Tailwind presentation classes per highlight color (for library UI cards).
// The canonical color list is defined in shared/lib/highlight-palette.ts.
const COLOR_TAILWIND: Record<
    HighlightColorName,
    { quoteBarGradient: string; dotClass: string; swatchClass: string }
> = {
    yellow: {
        quoteBarGradient: 'from-amber-400 to-amber-600',
        dotClass: 'bg-yellow-400',
        swatchClass: 'bg-yellow-500 dark:bg-yellow-400',
    },
    red: {
        quoteBarGradient: 'from-red-400 to-red-600',
        dotClass: 'bg-red-400',
        swatchClass: 'bg-red-500 dark:bg-red-400',
    },
    orange: {
        quoteBarGradient: 'from-orange-400 to-orange-600',
        dotClass: 'bg-orange-400',
        swatchClass: 'bg-orange-500 dark:bg-orange-400',
    },
    green: {
        quoteBarGradient: 'from-emerald-400 to-emerald-600',
        dotClass: 'bg-emerald-400',
        swatchClass: 'bg-emerald-500 dark:bg-emerald-400',
    },
    olive: {
        quoteBarGradient: 'from-lime-400 to-lime-600',
        dotClass: 'bg-lime-500',
        swatchClass: 'bg-lime-500 dark:bg-lime-400',
    },
    cyan: {
        quoteBarGradient: 'from-cyan-400 to-cyan-600',
        dotClass: 'bg-cyan-400',
        swatchClass: 'bg-cyan-500 dark:bg-cyan-400',
    },
    blue: {
        quoteBarGradient: 'from-blue-400 to-blue-600',
        dotClass: 'bg-blue-400',
        swatchClass: 'bg-blue-500 dark:bg-blue-400',
    },
    purple: {
        quoteBarGradient: 'from-purple-400 to-purple-600',
        dotClass: 'bg-purple-400',
        swatchClass: 'bg-purple-500 dark:bg-purple-400',
    },
    gray: {
        quoteBarGradient: 'from-gray-400 to-gray-600',
        dotClass: 'bg-gray-400',
        swatchClass: 'bg-gray-500 dark:bg-gray-400',
    },
};

export const HIGHLIGHT_COLORS = HIGHLIGHT_COLOR_NAMES.map((name) => ({
    name,
    ...COLOR_TAILWIND[name],
}));

export const DRAWER_TYPES = [
    {
        name: 'lighten',
        labelKey: 'highlight-drawer.lighten',
        icon: LuHighlighter,
        sampleClass: 'bg-amber-300/60 dark:bg-amber-400/40',
    },
    {
        name: 'underscore',
        labelKey: 'highlight-drawer.underscore',
        icon: LuUnderline,
        sampleClass:
            'underline underline-offset-2 decoration-2 decoration-current',
    },
    {
        name: 'strikeout',
        labelKey: 'highlight-drawer.strikeout',
        icon: LuStrikethrough,
        sampleClass: 'line-through decoration-2 decoration-current',
    },
    {
        name: 'invert',
        labelKey: 'highlight-drawer.invert',
        icon: LuContrast,
        sampleClass:
            'bg-gray-800 text-white dark:bg-white dark:text-gray-900 px-1 rounded-sm',
    },
] as const;

export const DRAWER_ICONS: Record<string, IconType> = Object.fromEntries(
    DRAWER_TYPES.map((d) => [d.name, d.icon]),
) as Record<string, IconType>;

// Rainbow gradient used for unrecognized (custom-patched) KOReader colors.
const UNKNOWN_DOT_CLASS =
    'bg-linear-to-br from-red-400 via-green-400 to-violet-400';
const UNKNOWN_QUOTE_BAR_GRADIENT =
    'from-red-400 via-green-400 to-violet-400';

// Lookup helpers derived from the arrays above.
const COLOR_DOT_MAP = Object.fromEntries(
    HIGHLIGHT_COLORS.map((c) => [c.name, c.dotClass]),
) as Record<string, string>;

const COLOR_QUOTE_BAR_MAP = Object.fromEntries(
    HIGHLIGHT_COLORS.map((c) => [c.name, c.quoteBarGradient]),
) as Record<string, string>;

export function colorDotClass(color: string | null | undefined): string {
    if (!color) return COLOR_DOT_MAP[DEFAULT_HIGHLIGHT_COLOR];
    return COLOR_DOT_MAP[color] ?? UNKNOWN_DOT_CLASS;
}

export function colorQuoteBarGradient(
    color: string | null | undefined,
): string {
    if (!color) return COLOR_QUOTE_BAR_MAP[DEFAULT_HIGHLIGHT_COLOR];
    return COLOR_QUOTE_BAR_MAP[color] ?? UNKNOWN_QUOTE_BAR_GRADIENT;
}
