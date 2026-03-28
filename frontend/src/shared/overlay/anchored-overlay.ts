export type OverlayPlacement = 'top' | 'bottom' | 'left' | 'right';

export type OverlayAlignment = 'center' | 'start' | 'end';

export type OverlayAlignmentOption =
    | OverlayAlignment
    | Partial<Record<OverlayPlacement, OverlayAlignment>>;

export type ViewportInset = {
    top?: number;
    bottom?: number;
    left?: number;
    right?: number;
};

export type OverlayPositioningOptions = {
    padding?: number;
    gap?: number;
    arrowSize?: number;
    arrowPadding?: number;
    placements?: OverlayPlacement[];
    alignment?: OverlayAlignmentOption;
    viewportInset?: ViewportInset;
};

type OverlayPositionResult = {
    placement: OverlayPlacement;
    top: number;
    left: number;
    arrowX: number;
    arrowY: number;
    anchorOffScreen: boolean;
};

const DEFAULT_PLACEMENTS: OverlayPlacement[] = [
    'top',
    'bottom',
    'right',
    'left',
];
const DEFAULT_PLACEMENT_CLASS_PREFIX = 'tooltip-';
const DEFAULT_PLACEMENT_CLASSES = [
    'tooltip-top',
    'tooltip-bottom',
    'tooltip-left',
    'tooltip-right',
];

function isAnchorOffScreen(
    anchorRect: DOMRect,
    inset: ViewportInset = {},
): boolean {
    const chromeTop = inset.top ?? 0;
    const chromeBottom = window.innerHeight - (inset.bottom ?? 0);
    const chromeLeft = inset.left ?? 0;
    const chromeRight = window.innerWidth - (inset.right ?? 0);

    // The anchor is off-screen when it straddles a chrome boundary (one edge
    // in the content area, one edge behind chrome). Chrome-resident anchors
    // like navbar buttons have both edges within chrome and are excluded.
    return (
        (anchorRect.top < chromeTop && anchorRect.bottom > chromeTop) ||
        (anchorRect.bottom > chromeBottom && anchorRect.top < chromeBottom) ||
        (anchorRect.left < chromeLeft && anchorRect.right > chromeLeft) ||
        (anchorRect.right > chromeRight && anchorRect.left < chromeRight) ||
        anchorRect.bottom <= 0 ||
        anchorRect.top >= window.innerHeight ||
        anchorRect.right <= 0 ||
        anchorRect.left >= window.innerWidth
    );
}

function placementClassesForPrefix(prefix: string): string[] {
    if (prefix === DEFAULT_PLACEMENT_CLASS_PREFIX) {
        return DEFAULT_PLACEMENT_CLASSES;
    }

    return [
        `${prefix}top`,
        `${prefix}bottom`,
        `${prefix}left`,
        `${prefix}right`,
    ];
}

export function computeOverlayPosition(
    anchorRect: DOMRect,
    overlayRect: DOMRect,
    viewportWidth: number,
    viewportHeight: number,
    options: OverlayPositioningOptions = {},
): OverlayPositionResult {
    const padding = options.padding ?? 8;
    const gap = options.gap ?? 8;
    const arrowSize = options.arrowSize ?? 6;
    const alignmentOption = options.alignment ?? 'center';
    const placements = options.placements ?? DEFAULT_PLACEMENTS;
    const inset = options.viewportInset;
    const rawInsetTop = inset?.top ?? 0;
    const rawInsetBottom = inset?.bottom ?? 0;
    const rawInsetLeft = inset?.left ?? 0;
    const rawInsetRight = inset?.right ?? 0;

    // When the anchor is inside chrome (e.g. a navbar button), clamp the
    // effective inset to the anchor's edge so the overlay is placed adjacent
    // to the anchor instead of being pushed into the content area.
    const insetTop = Math.max(0, Math.min(rawInsetTop, anchorRect.top));
    const insetBottom = Math.max(
        0,
        Math.min(rawInsetBottom, viewportHeight - anchorRect.bottom),
    );
    const insetLeft = Math.max(0, Math.min(rawInsetLeft, anchorRect.left));
    const insetRight = Math.max(
        0,
        Math.min(rawInsetRight, viewportWidth - anchorRect.right),
    );

    const anchorCenterX = anchorRect.left + anchorRect.width / 2;
    const anchorCenterY = anchorRect.top + anchorRect.height / 2;

    const spaceTop = anchorRect.top - insetTop;
    const spaceBottom = viewportHeight - anchorRect.bottom - insetBottom;
    const spaceLeft = anchorRect.left - insetLeft;
    const spaceRight = viewportWidth - anchorRect.right - insetRight;

    const requiredVertical = overlayRect.height + gap + arrowSize;
    const requiredHorizontal = overlayRect.width + gap + arrowSize;

    const candidates = placements.map((placement) => {
        if (placement === 'top') {
            return {
                placement,
                score:
                    spaceTop >= requiredVertical
                        ? 10000 + spaceTop
                        : spaceTop - requiredVertical,
            };
        }

        if (placement === 'bottom') {
            return {
                placement,
                score:
                    spaceBottom >= requiredVertical
                        ? 10000 + spaceBottom
                        : spaceBottom - requiredVertical,
            };
        }

        if (placement === 'right') {
            return {
                placement,
                score:
                    spaceRight >= requiredHorizontal
                        ? 10000 + spaceRight
                        : spaceRight - requiredHorizontal,
            };
        }

        return {
            placement,
            score:
                spaceLeft >= requiredHorizontal
                    ? 10000 + spaceLeft
                    : spaceLeft - requiredHorizontal,
        };
    });

    candidates.sort((left, right) => right.score - left.score);
    const placement = candidates[0]?.placement ?? 'top';
    const alignment =
        typeof alignmentOption === 'string'
            ? alignmentOption
            : (alignmentOption[placement] ?? 'center');

    let top = 0;
    let left = 0;

    if (placement === 'top' || placement === 'bottom') {
        top =
            placement === 'top'
                ? anchorRect.top - overlayRect.height - gap - arrowSize
                : anchorRect.bottom + gap + arrowSize;

        if (alignment === 'start') {
            left = anchorRect.left;
        } else if (alignment === 'end') {
            left = anchorRect.right - overlayRect.width;
        } else {
            left = anchorCenterX - overlayRect.width / 2;
        }
    } else {
        left =
            placement === 'right'
                ? anchorRect.right + gap + arrowSize
                : anchorRect.left - overlayRect.width - gap - arrowSize;

        if (alignment === 'start') {
            top = anchorRect.top;
        } else if (alignment === 'end') {
            top = anchorRect.bottom - overlayRect.height;
        } else {
            top = anchorCenterY - overlayRect.height / 2;
        }
    }

    const minLeft = padding + insetLeft;
    const minTop = padding + insetTop;
    const maxLeft = Math.max(
        minLeft,
        viewportWidth - overlayRect.width - padding - insetRight,
    );
    const maxTop = Math.max(
        minTop,
        viewportHeight - overlayRect.height - padding - insetBottom,
    );

    left = Math.min(Math.max(left, minLeft), maxLeft);
    top = Math.min(Math.max(top, minTop), maxTop);

    const arrowClampPadding = options.arrowPadding ?? arrowSize + 6;
    const arrowX = Math.min(
        Math.max(anchorCenterX - left, arrowClampPadding),
        Math.max(arrowClampPadding, overlayRect.width - arrowClampPadding),
    );
    const arrowY = Math.min(
        Math.max(anchorCenterY - top, arrowClampPadding),
        Math.max(arrowClampPadding, overlayRect.height - arrowClampPadding),
    );

    return {
        placement,
        top,
        left,
        arrowX,
        arrowY,
        anchorOffScreen: isAnchorOffScreen(anchorRect, inset),
    };
}

export function getShellInset(): ViewportInset {
    const inset: ViewportInset = {};

    const headerHeight = parseFloat(
        getComputedStyle(document.documentElement).getPropertyValue(
            '--header-height',
        ),
    );
    if (headerHeight > 0) {
        inset.top = headerHeight;
    }

    const bottomNav = document.querySelector<HTMLElement>('[data-shell-nav]');
    if (bottomNav && getComputedStyle(bottomNav).display !== 'none') {
        const bottom =
            window.innerHeight - bottomNav.getBoundingClientRect().top;
        if (bottom > 0) {
            inset.bottom = bottom;
        }
    }

    return inset;
}

type AnchoredOverlayOptions = OverlayPositioningOptions & {
    className: string;
    role?: string;
    hideClassName?: string;
    contentClassName?: string;
    placementClassPrefix?: string;
    maxWidthPadding?: number;
    hideOnOutsideClick?: boolean;
    updateContent?: (root: HTMLElement, content: string) => void;
    onVisibilityChange?: (visible: boolean, anchor: HTMLElement | null) => void;
};

export class AnchoredOverlay {
    private readonly root: HTMLElement;
    private readonly contentElement: HTMLElement | null;
    private readonly options: AnchoredOverlayOptions;
    private runtimeGap: number | undefined;
    private anchor: HTMLElement | null = null;
    private visible = false;
    private listenersBound = false;

    private readonly handleDocumentClick = (event: MouseEvent): void => {
        if (!this.options.hideOnOutsideClick || !this.visible || !this.anchor) {
            return;
        }

        const target = event.target instanceof Node ? event.target : null;
        if (!target) {
            return;
        }

        if (this.anchor.contains(target) || this.root.contains(target)) {
            return;
        }

        this.hide();
    };

    private readonly handleScrollOrResize = (): void => {
        this.refreshPosition();
    };

    constructor(options: AnchoredOverlayOptions) {
        this.options = options;

        const hideClassName = options.hideClassName ?? 'hidden';

        this.root = document.createElement('div');
        this.root.className = `${options.className} ${hideClassName}`.trim();
        this.root.setAttribute('role', options.role ?? 'tooltip');

        if (options.contentClassName) {
            const content = document.createElement('div');
            content.className = options.contentClassName;
            this.root.appendChild(content);
            this.contentElement = content;
        } else {
            this.contentElement = null;
        }

        document.body.appendChild(this.root);
    }

    show(
        anchor: HTMLElement,
        content: string,
        options?: { gap?: number },
    ): void {
        this.anchor = anchor;
        this.runtimeGap = options?.gap;
        this.setContent(content);

        this.root.classList.remove(this.options.hideClassName ?? 'hidden');
        this.visible = true;
        this.ensureListeners();
        this.refreshPosition();
        this.options.onVisibilityChange?.(true, this.anchor);
    }

    hide(): void {
        const previousAnchor = this.anchor;
        this.root.classList.add(this.options.hideClassName ?? 'hidden');
        this.visible = false;
        this.anchor = null;
        this.runtimeGap = undefined;

        const placementClassPrefix =
            this.options.placementClassPrefix ?? DEFAULT_PLACEMENT_CLASS_PREFIX;
        this.root.classList.remove(
            ...placementClassesForPrefix(placementClassPrefix),
        );
        this.options.onVisibilityChange?.(false, previousAnchor);
    }

    refreshPosition(): void {
        if (!this.visible || !this.anchor) {
            return;
        }

        if (!this.anchor.isConnected) {
            this.hide();
            return;
        }

        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;
        const maxWidthPadding =
            this.options.maxWidthPadding ?? this.options.padding ?? 8;

        this.root.style.position = 'fixed';
        this.root.style.maxWidth = `${Math.max(0, viewportWidth - maxWidthPadding * 2)}px`;

        const previousVisibility = this.root.style.visibility;
        this.root.style.visibility = 'hidden';
        this.root.classList.remove(this.options.hideClassName ?? 'hidden');
        this.root.style.top = '0px';
        this.root.style.left = '0px';
        const overlayRect = this.root.getBoundingClientRect();
        this.root.style.visibility = previousVisibility;

        const position = computeOverlayPosition(
            this.anchor.getBoundingClientRect(),
            overlayRect,
            viewportWidth,
            viewportHeight,
            {
                padding: this.options.padding,
                gap: this.runtimeGap ?? this.options.gap,
                arrowSize: this.options.arrowSize,
                placements: this.options.placements,
                alignment: this.options.alignment,
                viewportInset: getShellInset(),
            },
        );

        if (position.anchorOffScreen) {
            this.hide();
            return;
        }

        this.root.style.top = `${position.top}px`;
        this.root.style.left = `${position.left}px`;
        this.root.style.setProperty('--arrow-x', `${position.arrowX}px`);
        this.root.style.setProperty('--arrow-y', `${position.arrowY}px`);

        const placementClassPrefix =
            this.options.placementClassPrefix ?? DEFAULT_PLACEMENT_CLASS_PREFIX;
        this.root.classList.remove(
            ...placementClassesForPrefix(placementClassPrefix),
        );
        this.root.classList.add(`${placementClassPrefix}${position.placement}`);
    }

    private setContent(content: string): void {
        if (this.options.updateContent) {
            this.options.updateContent(this.root, content);
            return;
        }

        if (this.contentElement) {
            this.contentElement.textContent = content;
            return;
        }

        this.root.textContent = content;
    }

    private ensureListeners(): void {
        if (this.listenersBound) {
            return;
        }

        this.listenersBound = true;

        if (this.options.hideOnOutsideClick) {
            document.addEventListener('click', this.handleDocumentClick);
        }

        document.addEventListener('scroll', this.handleScrollOrResize, {
            passive: true,
            capture: true,
        });
        window.addEventListener('resize', this.handleScrollOrResize, {
            passive: true,
        });
    }
}
