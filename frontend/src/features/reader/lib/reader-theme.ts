import type { FoliateView } from '../model/reader-model';

export const READER_LAYOUT_SETTINGS = {
    margin: '24px',
    gap: '4.5%',
    maxInlineSize: '840px',
    topInset: '8px',
} as const;

export const DEFAULT_READER_FONT_SIZE = 112;
export const DEFAULT_READER_LINE_SPACING = 1.5;

const READER_THEME_COLORS = {
    light: {
        background: '#ffffff',
        text: '#111827',
        link: '#0369a1',
    },
    dark: {
        background: '#0a0f1a',
        text: '#e5e7eb',
        link: '#7dd3fc',
    },
} as const;

function buildReaderBaseStyles(fontSize: number, lineSpacing: number): string {
    return `
@namespace epub "http://www.idpf.org/2007/ops";

html {
    font-size: ${fontSize}% !important;
    line-height: ${lineSpacing} !important;
}

p {
    line-height: ${lineSpacing} !important;
}

html,
body {
    margin: 0 !important;
    padding: 0 !important;
}
`;
}

export function buildReaderThemeStyles(isDarkMode: boolean): string {
    const colors = isDarkMode
        ? READER_THEME_COLORS.dark
        : READER_THEME_COLORS.light;

    const darkModeOverrides = isDarkMode
        ? `
body :where(*):not(img):not(image):not(video):not(audio):not(canvas):not(svg):not(svg *) {
    background-color: transparent !important;
    background-image: none !important;
}
`
        : '';

    return `
html,
body {
    color-scheme: ${isDarkMode ? 'dark' : 'light'};
    background: ${colors.background} !important;
    color: ${colors.text} !important;
}

body :where(*):not(img):not(image):not(video):not(audio):not(canvas):not(svg):not(svg *) {
    color: inherit !important;
    -webkit-text-fill-color: currentColor !important;
}

a,
a:link,
a:visited {
    color: ${colors.link} !important;
}

${darkModeOverrides}`;
}

export function applyReaderPresentation(
    view: FoliateView,
    fontSize = DEFAULT_READER_FONT_SIZE,
    lineSpacing = DEFAULT_READER_LINE_SPACING,
): void {
    const renderer = view.renderer;
    if (!renderer) {
        return;
    }

    renderer.setAttribute('margin', READER_LAYOUT_SETTINGS.margin);
    renderer.setAttribute('gap', READER_LAYOUT_SETTINGS.gap);
    renderer.setAttribute(
        'max-inline-size',
        READER_LAYOUT_SETTINGS.maxInlineSize,
    );

    if (!view.isFixedLayout) {
        renderer.style.boxSizing = 'border-box';
        renderer.style.paddingTop = READER_LAYOUT_SETTINGS.topInset;

        renderer.setStyles?.([
            buildReaderBaseStyles(fontSize, lineSpacing),
            buildReaderThemeStyles(
                document.documentElement.classList.contains('dark'),
            ),
        ]);
    }
}
