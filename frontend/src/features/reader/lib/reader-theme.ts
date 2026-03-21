import type { LibraryReaderPresentation } from '../../library/api/library-data';
import { resolveReaderFontOverride } from './reader-fonts';
import { toFiniteNonNegativeNumber } from './reader-presentation';
import type { FoliateView } from '../model/reader-model';

export const READER_LAYOUT_SETTINGS = {
    gap: '4.5%',
    maxInlineSize: '840px',
} as const;

export const DEFAULT_READER_FONT_SIZE_PT = 22;
export const DEFAULT_READER_LINE_SPACING = 1.3;
export const DEFAULT_READER_LEFT_MARGIN = 12;
export const DEFAULT_READER_RIGHT_MARGIN = 12;
export const DEFAULT_READER_TOP_MARGIN = 20;
export const DEFAULT_READER_BOTTOM_MARGIN = 20;

const CSS_ROOT_FONT_SIZE_PX = 16;
const KOREADER_TO_BROWSER_FONT_SCALE = 0.95;
const VIEW_FONT_OVERRIDE_REQUESTS = new WeakMap<FoliateView, number>();
const VIEW_FONT_OVERRIDE_CACHE = new WeakMap<
    FoliateView,
    {
        fontFaceKey: string;
        override: {
            fontFamilyCssValue: string;
            fontFaceCss: string;
        } | null;
    }
>();
let NEXT_FONT_OVERRIDE_REQUEST_ID = 1;

type ReaderHyphenationMode = 'auto' | 'none' | null;
type ReaderFloatingPunctuationMode = 'first last' | 'none' | null;

type ResolvedReaderLayout = {
    rendererMargin: string;
    gap: string;
    maxInlineSize: string;
    contentPaddingLeft: string;
    contentPaddingRight: string;
    hyphenation: ReaderHyphenationMode;
    floatingPunctuation: ReaderFloatingPunctuationMode;
};

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

export function mapKoReaderFontSizePtToCssPercent(
    fontSizePt: number,
    sizeScale = 1,
): number {
    const normalizedFontSizePt =
        Number.isFinite(fontSizePt) && fontSizePt > 0
            ? fontSizePt
            : DEFAULT_READER_FONT_SIZE_PT;
    const normalizedSizeScale =
        Number.isFinite(sizeScale) && sizeScale > 0 ? sizeScale : 1;
    const scaledFontSizePx = Math.max(
        1,
        Math.ceil(normalizedFontSizePt * normalizedSizeScale),
    );
    const calibratedFontSizePx =
        scaledFontSizePx * KOREADER_TO_BROWSER_FONT_SCALE;

    return Number(
        ((calibratedFontSizePx / CSS_ROOT_FONT_SIZE_PX) * 100).toFixed(2),
    );
}

export function mapKoReaderLineSpacingPercentToCssLineHeight(
    lineSpacingPercent: number,
): number {
    return Number((lineSpacingPercent / 100).toFixed(2));
}

function resolveHorizontalMarginsPx(
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

    return [Math.round(left), Math.round(right)];
}

function resolveHyphenationMode(
    presentation: LibraryReaderPresentation | null | undefined,
): ReaderHyphenationMode {
    if (presentation?.hyphenation === true) {
        return 'auto';
    }

    if (presentation?.hyphenation === false) {
        return 'none';
    }

    return null;
}

function resolveFloatingPunctuationMode(
    presentation: LibraryReaderPresentation | null | undefined,
): ReaderFloatingPunctuationMode {
    if (presentation?.floating_punctuation === true) {
        return 'first last';
    }

    if (presentation?.floating_punctuation === false) {
        return 'none';
    }

    return null;
}

function resolveReaderLayout(
    presentation: LibraryReaderPresentation | null | undefined,
): ResolvedReaderLayout {
    const [leftMarginPx, rightMarginPx] =
        resolveHorizontalMarginsPx(presentation);

    return {
        rendererMargin: '0px',
        gap: READER_LAYOUT_SETTINGS.gap,
        maxInlineSize: READER_LAYOUT_SETTINGS.maxInlineSize,
        contentPaddingLeft: `${Math.max(0, leftMarginPx)}px`,
        contentPaddingRight: `${Math.max(0, rightMarginPx)}px`,
        hyphenation: resolveHyphenationMode(presentation),
        floatingPunctuation: resolveFloatingPunctuationMode(presentation),
    };
}

type ReaderFontOverride = {
    fontFamilyCssValue: string;
    fontFaceCss: string;
} | null;

function normalizeFontFaceKey(fontFace: string | null | undefined): string {
    return fontFace?.trim() ?? '';
}

function buildReaderBaseStyles(
    fontSizePercent: number,
    lineSpacing: number,
    layout: ResolvedReaderLayout,
    fontOverride: ReaderFontOverride = null,
): string {
    const hyphenationStyles =
        layout.hyphenation === null
            ? ''
            : `
html,
body,
p,
li,
dd,
dt,
blockquote {
    -webkit-hyphens: ${layout.hyphenation} !important;
    hyphens: ${layout.hyphenation} !important;
}
`;

    const floatingPunctuationStyles =
        layout.floatingPunctuation === null
            ? ''
            : `
html,
body,
p,
li,
dd,
dt,
blockquote {
    hanging-punctuation: ${layout.floatingPunctuation} !important;
}
`;

    const fontOverrideStyles = fontOverride
        ? `
${fontOverride.fontFaceCss}

html,
body,
body :where(*):not(pre):not(code):not(kbd):not(samp):not(var):not(math):not(svg):not(svg *):not(img):not(image):not(video):not(audio):not(canvas) {
    font-family: ${fontOverride.fontFamilyCssValue} !important;
}
`
        : '';

    return `
@namespace epub "http://www.idpf.org/2007/ops";

html {
    font-size: ${fontSizePercent}% !important;
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

body {
    box-sizing: border-box !important;
    padding-top: 0 !important;
    padding-right: ${layout.contentPaddingRight} !important;
    padding-bottom: 0 !important;
    padding-left: ${layout.contentPaddingLeft} !important;
    -webkit-box-decoration-break: clone;
    box-decoration-break: clone;
}

${hyphenationStyles}
${floatingPunctuationStyles}
${fontOverrideStyles}
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
    fontSizePt = DEFAULT_READER_FONT_SIZE_PT,
    lineSpacing = DEFAULT_READER_LINE_SPACING,
    presentation: LibraryReaderPresentation | null | undefined = null,
): void {
    const renderer = view.renderer;
    if (!renderer) {
        return;
    }

    const layout = resolveReaderLayout(presentation);
    const normalizedFontSizePt =
        Number.isFinite(fontSizePt) && fontSizePt > 0
            ? fontSizePt
            : DEFAULT_READER_FONT_SIZE_PT;
    const normalizedLineSpacing =
        Number.isFinite(lineSpacing) && lineSpacing > 0
            ? lineSpacing
            : DEFAULT_READER_LINE_SPACING;
    const fontSizePercent =
        mapKoReaderFontSizePtToCssPercent(normalizedFontSizePt);

    renderer.setAttribute('margin', layout.rendererMargin);
    renderer.setAttribute('gap', layout.gap);
    renderer.setAttribute('max-inline-size', layout.maxInlineSize);

    if (view.isFixedLayout) {
        VIEW_FONT_OVERRIDE_REQUESTS.delete(view);
        return;
    }

    const applyStyles = (fontOverride: ReaderFontOverride): void => {
        if (view.renderer !== renderer) {
            return;
        }

        renderer.setStyles?.([
            buildReaderBaseStyles(
                fontSizePercent,
                normalizedLineSpacing,
                layout,
                fontOverride,
            ),
            buildReaderThemeStyles(
                document.documentElement.classList.contains('dark'),
            ),
        ]);
    };

    if (presentation?.embedded_fonts === false) {
        const fontFaceKey = normalizeFontFaceKey(presentation?.font_face);
        const cachedOverride = VIEW_FONT_OVERRIDE_CACHE.get(view);
        const requestId = NEXT_FONT_OVERRIDE_REQUEST_ID;
        NEXT_FONT_OVERRIDE_REQUEST_ID += 1;
        VIEW_FONT_OVERRIDE_REQUESTS.set(view, requestId);

        if (cachedOverride?.fontFaceKey === fontFaceKey) {
            applyStyles(cachedOverride.override);
            return;
        }

        applyStyles(null);

        void resolveReaderFontOverride(presentation?.font_face)
            .then((fontOverride) => {
                if (VIEW_FONT_OVERRIDE_REQUESTS.get(view) !== requestId) {
                    return;
                }

                VIEW_FONT_OVERRIDE_CACHE.set(view, {
                    fontFaceKey,
                    override: fontOverride,
                });
                applyStyles(fontOverride);
            })
            .catch(() => {
                if (VIEW_FONT_OVERRIDE_REQUESTS.get(view) !== requestId) {
                    return;
                }

                applyStyles(null);
            });

        return;
    }

    VIEW_FONT_OVERRIDE_REQUESTS.delete(view);
    applyStyles(null);
}
