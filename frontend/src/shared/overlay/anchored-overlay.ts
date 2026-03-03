export type OverlayPlacement = 'top' | 'bottom' | 'left' | 'right';

export type OverlayPositioningOptions = {
    padding?: number;
    gap?: number;
    arrowSize?: number;
    placementOrder?: OverlayPlacement[];
};

type OverlayPositionResult = {
    placement: OverlayPlacement;
    top: number;
    left: number;
    arrowX: number;
    arrowY: number;
};

const DEFAULT_PLACEMENT_ORDER: OverlayPlacement[] = ['top', 'bottom', 'right', 'left'];
const DEFAULT_PLACEMENT_CLASS_PREFIX = 'tooltip-';
const DEFAULT_PLACEMENT_CLASSES = [
    'tooltip-top',
    'tooltip-bottom',
    'tooltip-left',
    'tooltip-right',
];

function placementClassesForPrefix(prefix: string): string[] {
    if (prefix === DEFAULT_PLACEMENT_CLASS_PREFIX) {
        return DEFAULT_PLACEMENT_CLASSES;
    }

    return [`${prefix}top`, `${prefix}bottom`, `${prefix}left`, `${prefix}right`];
}

function normalizePlacementOrder(order: OverlayPlacement[] | undefined): OverlayPlacement[] {
    if (!order || order.length === 0) {
        return DEFAULT_PLACEMENT_ORDER;
    }

    const seen = new Set<OverlayPlacement>();
    const normalized: OverlayPlacement[] = [];

    order.forEach((placement) => {
        if (!seen.has(placement)) {
            normalized.push(placement);
            seen.add(placement);
        }
    });

    DEFAULT_PLACEMENT_ORDER.forEach((placement) => {
        if (!seen.has(placement)) {
            normalized.push(placement);
        }
    });

    return normalized;
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
    const placementOrder = normalizePlacementOrder(options.placementOrder);

    const anchorCenterX = anchorRect.left + anchorRect.width / 2;
    const anchorCenterY = anchorRect.top + anchorRect.height / 2;

    const spaceTop = anchorRect.top;
    const spaceBottom = viewportHeight - anchorRect.bottom;
    const spaceLeft = anchorRect.left;
    const spaceRight = viewportWidth - anchorRect.right;

    const requiredVertical = overlayRect.height + gap + arrowSize;
    const requiredHorizontal = overlayRect.width + gap + arrowSize;

    const candidates = placementOrder.map((placement) => {
        if (placement === 'top') {
            return {
                placement,
                score:
                    spaceTop >= requiredVertical ? 10000 + spaceTop : spaceTop - requiredVertical,
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

    let top = 0;
    let left = 0;

    if (placement === 'top') {
        top = anchorRect.top - overlayRect.height - gap - arrowSize;
        left = anchorCenterX - overlayRect.width / 2;
    } else if (placement === 'bottom') {
        top = anchorRect.bottom + gap + arrowSize;
        left = anchorCenterX - overlayRect.width / 2;
    } else if (placement === 'right') {
        top = anchorCenterY - overlayRect.height / 2;
        left = anchorRect.right + gap + arrowSize;
    } else {
        top = anchorCenterY - overlayRect.height / 2;
        left = anchorRect.left - overlayRect.width - gap - arrowSize;
    }

    const maxLeft = Math.max(padding, viewportWidth - overlayRect.width - padding);
    const maxTop = Math.max(padding, viewportHeight - overlayRect.height - padding);

    left = Math.min(Math.max(left, padding), maxLeft);
    top = Math.min(Math.max(top, padding), maxTop);

    const arrowClampPadding = arrowSize + 6;
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
    };
}

export type AnchoredOverlayOptions = OverlayPositioningOptions & {
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

export type AnchoredOverlayRuntimeOptions = Pick<
    AnchoredOverlayOptions,
    'padding' | 'gap' | 'arrowSize' | 'placementOrder' | 'maxWidthPadding'
>;

export class AnchoredOverlay {
    private readonly root: HTMLElement;
    private readonly contentElement: HTMLElement | null;
    private readonly options: AnchoredOverlayOptions;
    private runtimeOptions: Partial<AnchoredOverlayRuntimeOptions> = {};
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
        runtimeOptions?: Partial<AnchoredOverlayRuntimeOptions>,
    ): void {
        this.anchor = anchor;
        this.runtimeOptions = runtimeOptions ?? {};
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
        this.runtimeOptions = {};

        const placementClassPrefix =
            this.options.placementClassPrefix ?? DEFAULT_PLACEMENT_CLASS_PREFIX;
        this.root.classList.remove(...placementClassesForPrefix(placementClassPrefix));
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
            this.runtimeOptions.maxWidthPadding ??
            this.options.maxWidthPadding ??
            this.runtimeOptions.padding ??
            this.options.padding ??
            8;

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
                padding: this.runtimeOptions.padding ?? this.options.padding,
                gap: this.runtimeOptions.gap ?? this.options.gap,
                arrowSize: this.runtimeOptions.arrowSize ?? this.options.arrowSize,
                placementOrder: this.runtimeOptions.placementOrder ?? this.options.placementOrder,
            },
        );

        this.root.style.top = `${position.top}px`;
        this.root.style.left = `${position.left}px`;
        this.root.style.setProperty('--arrow-x', `${position.arrowX}px`);
        this.root.style.setProperty('--arrow-y', `${position.arrowY}px`);

        const placementClassPrefix =
            this.options.placementClassPrefix ?? DEFAULT_PLACEMENT_CLASS_PREFIX;
        this.root.classList.remove(...placementClassesForPrefix(placementClassPrefix));
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
        window.addEventListener('resize', this.handleScrollOrResize, { passive: true });
    }
}
