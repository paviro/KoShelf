import {
    DEFAULT_HIGHLIGHT_COLOR,
    HIGHLIGHT_HEX,
    type HighlightColorName,
} from '../../../shared/lib/highlight-palette';

export function highlightColor(
    name: string | null | undefined,
    isDark: boolean,
): string {
    const key = (name ?? DEFAULT_HIGHLIGHT_COLOR) as HighlightColorName;
    const pair = HIGHLIGHT_HEX[key] ?? HIGHLIGHT_HEX[DEFAULT_HIGHLIGHT_COLOR];
    return isDark ? pair.dark : pair.light;
}
