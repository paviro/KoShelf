// WebKit-only repaint workaround for fixed / backdrop-filter headers.
//
// WebKit (especially when scrollY == 0) can fail to repaint the page
// until the next scroll. We simulate a tiny scroll nudge (1px)
// If the page isn't scrollable, we temporarily add a 2px spacer.

function isWebKitEngine(): boolean {
    // Heuristic UA sniff: only used for a WebKit repaint workaround.
    // Goal: include all iOS browsers (all are WebKit) + desktop Safari,
    // while avoiding desktop Blink browsers that also include "AppleWebKit" in UA.
    const ua = navigator.userAgent;
    const isAppleWebKit = /AppleWebKit/i.test(ua);
    const isIOS = /iP(hone|ad|od)/i.test(ua);
    const isMacSafari =
        /Macintosh/i.test(ua) && /Safari/i.test(ua) && !/Chrome|CriOS|Edg|OPR|Firefox/i.test(ua);

    return isAppleWebKit && (isIOS || isMacSafari);
}

export type WebkitRepaintHackBreakpointBucket = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';

function getTailwindBreakpointBucket(w: number): WebkitRepaintHackBreakpointBucket {
    // Tailwind defaults: sm=640, md=768, lg=1024, xl=1280, 2xl=1536
    if (w < 640) return 'xs';
    if (w < 768) return 'sm';
    if (w < 1024) return 'md';
    if (w < 1280) return 'lg';
    if (w < 1536) return 'xl';
    return '2xl';
}

let installed = false;

export function installWebkitResizeRepaintHack(): void {
    if (!isWebKitEngine()) return;
    if (installed) return;
    installed = true;

    // Log once so it's easy to confirm it's active.
    console.debug('[KoShelf] WebKit repaint hack active');

    let lastBucket: WebkitRepaintHackBreakpointBucket = getTailwindBreakpointBucket(window.innerWidth);
    let scheduled = false;

    const nudge = () => {
        // Only needed when at the very top (this is where WebKit gets stuck).
        if (window.scrollY !== 0) return;

        // Avoid jitter: only trigger when crossing responsive breakpoints.
        const currentBucket = getTailwindBreakpointBucket(window.innerWidth);
        if (currentBucket === lastBucket) return;
        lastBucket = currentBucket;

        // If the document isn't scrollable, Safari can't "nudge scroll" at all.
        // Temporarily add a tiny spacer to force scrollability, then remove it.
        const docEl = document.documentElement;
        const body = document.body;

        const prevDocOverflowY = docEl.style.overflowY;
        const prevBodyOverflowY = body.style.overflowY;

        const needsSpacer = docEl.scrollHeight <= window.innerHeight + 1;
        const spacer = needsSpacer ? document.createElement('div') : null;
        if (spacer) {
            spacer.setAttribute('data-safari-repaint-spacer', 'true');
            spacer.style.height = '2px';
            spacer.style.width = '1px';
            spacer.style.pointerEvents = 'none';
            body.appendChild(spacer);
        }

        // Ensure a scroll container exists during the nudge.
        docEl.style.overflowY = 'scroll';
        body.style.overflowY = 'scroll';

        requestAnimationFrame(() => {
            // Still at the top in this frame: nudge to force repaint.
            if (window.scrollY === 0) window.scrollBy(0, 1);

            requestAnimationFrame(() => {
                // If user scrolled during the hack, don't fight them.
                if (window.scrollY === 1) window.scrollTo(0, 0);

                spacer?.remove();
                docEl.style.overflowY = prevDocOverflowY;
                body.style.overflowY = prevBodyOverflowY;
            });
        });
    };

    const schedule = () => {
        if (scheduled) return;
        scheduled = true;
        requestAnimationFrame(() => {
            scheduled = false;
            nudge();
        });
    };

    window.addEventListener('resize', schedule, { passive: true });
    window.addEventListener('orientationchange', schedule, { passive: true });
    window.addEventListener('pageshow', schedule, { passive: true });

    // On iOS Safari, visualViewport resizes can happen without a window resize.
    window.visualViewport?.addEventListener('resize', schedule, { passive: true });
}

// Auto-install when included in the base bundle.
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => installWebkitResizeRepaintHack());
} else {
    installWebkitResizeRepaintHack();
}
