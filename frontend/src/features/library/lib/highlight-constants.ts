import {
    LuHighlighter,
    LuStrikethrough,
    LuType,
    LuUnderline,
} from 'react-icons/lu';
import type { IconType } from 'react-icons';

export const HIGHLIGHT_COLORS = [
    {
        name: 'yellow',
        quoteBarGradient: 'from-amber-400 to-amber-600',
        dotClass: 'bg-yellow-400',
        swatchClass: 'bg-yellow-500 dark:bg-yellow-400',
    },
    {
        name: 'red',
        quoteBarGradient: 'from-red-400 to-red-600',
        dotClass: 'bg-red-400',
        swatchClass: 'bg-red-500 dark:bg-red-400',
    },
    {
        name: 'orange',
        quoteBarGradient: 'from-orange-400 to-orange-600',
        dotClass: 'bg-orange-400',
        swatchClass: 'bg-orange-500 dark:bg-orange-400',
    },
    {
        name: 'green',
        quoteBarGradient: 'from-emerald-400 to-emerald-600',
        dotClass: 'bg-emerald-400',
        swatchClass: 'bg-emerald-500 dark:bg-emerald-400',
    },
    {
        name: 'olive',
        quoteBarGradient: 'from-lime-400 to-lime-600',
        dotClass: 'bg-lime-500',
        swatchClass: 'bg-lime-500 dark:bg-lime-400',
    },
    {
        name: 'cyan',
        quoteBarGradient: 'from-cyan-400 to-cyan-600',
        dotClass: 'bg-cyan-400',
        swatchClass: 'bg-cyan-500 dark:bg-cyan-400',
    },
    {
        name: 'blue',
        quoteBarGradient: 'from-blue-400 to-blue-600',
        dotClass: 'bg-blue-400',
        swatchClass: 'bg-blue-500 dark:bg-blue-400',
    },
    {
        name: 'purple',
        quoteBarGradient: 'from-purple-400 to-purple-600',
        dotClass: 'bg-purple-400',
        swatchClass: 'bg-purple-500 dark:bg-purple-400',
    },
    {
        name: 'gray',
        quoteBarGradient: 'from-gray-400 to-gray-600',
        dotClass: 'bg-gray-400',
        swatchClass: 'bg-gray-500 dark:bg-gray-400',
    },
] as const;

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
        icon: LuType,
        sampleClass:
            'bg-gray-800 text-white dark:bg-white dark:text-gray-900 px-1 rounded-sm',
    },
] as const;

export const DRAWER_ICONS: Record<string, IconType> = Object.fromEntries(
    DRAWER_TYPES.map((d) => [d.name, d.icon]),
) as Record<string, IconType>;

// Lookup helpers derived from the arrays above.
const COLOR_DOT_MAP = Object.fromEntries(
    HIGHLIGHT_COLORS.map((c) => [c.name, c.dotClass]),
) as Record<string, string>;

const COLOR_QUOTE_BAR_MAP = Object.fromEntries(
    HIGHLIGHT_COLORS.map((c) => [c.name, c.quoteBarGradient]),
) as Record<string, string>;

const DEFAULT_COLOR = 'yellow';

export function colorDotClass(color: string | null | undefined): string {
    return (
        COLOR_DOT_MAP[color ?? DEFAULT_COLOR] ?? COLOR_DOT_MAP[DEFAULT_COLOR]
    );
}

export function colorQuoteBarGradient(
    color: string | null | undefined,
): string {
    return (
        COLOR_QUOTE_BAR_MAP[color ?? DEFAULT_COLOR] ??
        COLOR_QUOTE_BAR_MAP[DEFAULT_COLOR]
    );
}
