/**
 * Canonical highlight color palette shared between the reader overlay engine
 * (which needs hex values) and the library UI (which uses Tailwind classes).
 *
 * Add or remove colors here — both systems derive from this list.
 */

export const HIGHLIGHT_COLOR_NAMES = [
    'yellow',
    'red',
    'orange',
    'green',
    'olive',
    'cyan',
    'blue',
    'purple',
    'gray',
] as const;

export type HighlightColorName = (typeof HIGHLIGHT_COLOR_NAMES)[number];

export const DEFAULT_HIGHLIGHT_COLOR: HighlightColorName = 'yellow';

type LightDarkPair = { light: string; dark: string };

export const HIGHLIGHT_HEX: Record<HighlightColorName, LightDarkPair> = {
    yellow: { light: '#eab308', dark: '#facc15' },
    red: { light: '#ef4444', dark: '#f87171' },
    orange: { light: '#f97316', dark: '#fb923c' },
    green: { light: '#10b981', dark: '#34d399' },
    olive: { light: '#84cc16', dark: '#a3e635' },
    cyan: { light: '#06b6d4', dark: '#22d3ee' },
    blue: { light: '#3b82f6', dark: '#60a5fa' },
    purple: { light: '#a855f7', dark: '#c084fc' },
    gray: { light: '#6b7280', dark: '#9ca3af' },
};
