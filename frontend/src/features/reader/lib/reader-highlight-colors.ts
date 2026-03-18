const LIGHT_COLORS: Record<string, string> = {
    red: '#ef4444',
    orange: '#f97316',
    yellow: '#eab308',
    green: '#10b981',
    olive: '#84cc16',
    cyan: '#06b6d4',
    blue: '#3b82f6',
    purple: '#a855f7',
    gray: '#6b7280',
};

const DARK_COLORS: Record<string, string> = {
    red: '#f87171',
    orange: '#fb923c',
    yellow: '#facc15',
    green: '#34d399',
    olive: '#a3e635',
    cyan: '#22d3ee',
    blue: '#60a5fa',
    purple: '#c084fc',
    gray: '#9ca3af',
};

const DEFAULT_LIGHT = '#eab308';
const DEFAULT_DARK = '#facc15';

export function highlightColor(
    name: string | null | undefined,
    isDark: boolean,
): string {
    const palette = isDark ? DARK_COLORS : LIGHT_COLORS;
    const fallback = isDark ? DEFAULT_DARK : DEFAULT_LIGHT;
    if (!name) return fallback;
    return palette[name] ?? fallback;
}
