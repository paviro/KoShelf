import { useEffect, useRef } from 'react';

import { api } from '../../../shared/api';
import type { LibraryDetailPreviewResponse } from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';

interface PreviewCardData {
    title: string;
    author: string;
    series: string;
    description: string;
}

interface LibraryCardElement extends HTMLElement {
    dataset: DOMStringMap & {
        libraryItemId?: string;
        libraryCollection?: string;
        libraryItemTitle?: string;
        libraryItemAuthors?: string;
        libraryItemSeries?: string;
    };
}

const SHOW_DELAY_MS = 160;
const HIDE_DELAY_MS = 90;
const FADE_DURATION_MS = 200;
const SCROLL_IDLE_MS = 140;
const PREVIEW_OFFSET_PX = 14;
const VIEWPORT_PADDING_PX = 10;
const PREVIEW_ELEMENT_ID = 'libraryHoverPreviewTooltip';

function supportsHoverPreview(): boolean {
    if (!window.matchMedia('(pointer: fine)').matches) {
        return false;
    }

    if (!window.matchMedia('(hover: hover)').matches) {
        return false;
    }

    if (window.matchMedia('(max-width: 1023px)').matches) {
        return false;
    }

    return true;
}

class HoverPreviewManager {
    private previewElement: HTMLElement | null = null;
    private targetCard: LibraryCardElement | null = null;
    private isVisible = false;
    private showTimeoutId: number | null = null;
    private hideTimeoutId: number | null = null;
    private fadeTimeoutId: number | null = null;
    private scrollIdleTimeoutId: number | null = null;
    private isUserScrolling = false;
    private hoveredCard: LibraryCardElement | null = null;
    private readonly detailsCache: Map<string, PreviewCardData>;
    private cardListeners: Array<() => void> = [];

    private readonly handleScroll = (): void => {
        this.markScrolling();
        if (!this.isVisible) {
            return;
        }
        this.hideWithAnimation();
    };

    private readonly handleResize = (): void => {
        if (!supportsHoverPreview()) {
            this.hideNow();
            return;
        }

        if (!this.previewElement || !this.targetCard || !this.isVisible) {
            return;
        }

        this.positionPreview(this.targetCard, this.previewElement);
    };

    private readonly handleEscapeKey = (event: KeyboardEvent): void => {
        if (event.key === 'Escape') {
            this.hideWithAnimation();
        }
    };

    constructor(detailsCache: Map<string, PreviewCardData>) {
        this.detailsCache = detailsCache;
    }

    init(selector: string): () => void {
        if (!supportsHoverPreview()) {
            return () => {};
        }

        this.ensurePreviewElement();

        const cards = document.querySelectorAll<LibraryCardElement>(selector);
        cards.forEach((card) => {
            const handleMouseEnter = (): void => {
                this.hoveredCard = card;
                void this.loadPreviewData(card);
                this.scheduleShow(card);
            };

            const handleMouseLeave = (event: MouseEvent): void => {
                const relatedTarget = event.relatedTarget;
                if (
                    relatedTarget instanceof Node &&
                    card.contains(relatedTarget)
                ) {
                    return;
                }

                if (this.hoveredCard === card) {
                    this.hoveredCard = null;
                }
                this.scheduleHide();
            };

            const handleFocusIn = (): void => {
                this.hoveredCard = card;
                void this.loadPreviewData(card);
                this.scheduleShow(card);
            };

            const handleFocusOut = (event: FocusEvent): void => {
                const relatedTarget = event.relatedTarget;
                if (
                    relatedTarget instanceof Node &&
                    card.contains(relatedTarget)
                ) {
                    return;
                }

                if (this.hoveredCard === card && !card.matches(':hover')) {
                    this.hoveredCard = null;
                }
                this.scheduleHide();
            };

            const handleClick = (): void => {
                this.hideNow();
            };

            card.addEventListener('mouseenter', handleMouseEnter);
            card.addEventListener('mouseleave', handleMouseLeave);
            card.addEventListener('focusin', handleFocusIn);
            card.addEventListener('focusout', handleFocusOut);
            card.addEventListener('click', handleClick);

            this.cardListeners.push(() => {
                card.removeEventListener('mouseenter', handleMouseEnter);
                card.removeEventListener('mouseleave', handleMouseLeave);
                card.removeEventListener('focusin', handleFocusIn);
                card.removeEventListener('focusout', handleFocusOut);
                card.removeEventListener('click', handleClick);
            });
        });

        window.addEventListener('scroll', this.handleScroll, {
            passive: true,
            capture: true,
        });
        window.addEventListener('resize', this.handleResize, { passive: true });
        document.addEventListener('keydown', this.handleEscapeKey);

        return () => {
            this.destroy();
        };
    }

    private destroy(): void {
        this.clearTimeouts();
        this.clearScrollIdleTimeout();
        this.hoveredCard = null;
        this.targetCard = null;
        this.isVisible = false;

        this.cardListeners.forEach((unbind) => unbind());
        this.cardListeners = [];

        window.removeEventListener('scroll', this.handleScroll, true);
        window.removeEventListener('resize', this.handleResize);
        document.removeEventListener('keydown', this.handleEscapeKey);

        if (this.previewElement?.parentNode) {
            this.previewElement.parentNode.removeChild(this.previewElement);
        }

        this.previewElement = null;
    }

    private hideNow(): void {
        this.clearTimeouts();
        this.clearScrollIdleTimeout();
        this.hoveredCard = null;

        if (!this.previewElement) {
            return;
        }

        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';
        this.previewElement.style.visibility = 'hidden';
        this.previewElement.classList.add('hidden');
        this.previewElement.setAttribute('aria-hidden', 'true');

        this.targetCard = null;
        this.isVisible = false;
    }

    private scheduleShow(card: LibraryCardElement): void {
        if (this.isUserScrolling) {
            return;
        }

        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();

        this.showTimeoutId = window.setTimeout(() => {
            void this.show(card);
        }, SHOW_DELAY_MS);
    }

    private scheduleHide(): void {
        this.clearShowTimeout();

        this.hideTimeoutId = window.setTimeout(() => {
            this.hideWithAnimation();
        }, HIDE_DELAY_MS);
    }

    private hideWithAnimation(): void {
        if (!this.previewElement) {
            return;
        }

        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();

        this.isVisible = false;
        this.targetCard = null;

        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';

        this.fadeTimeoutId = window.setTimeout(() => {
            if (!this.previewElement) {
                return;
            }

            this.previewElement.style.visibility = 'hidden';
            this.previewElement.classList.add('hidden');
            this.previewElement.setAttribute('aria-hidden', 'true');
            this.fadeTimeoutId = null;
        }, FADE_DURATION_MS);
    }

    private async show(card: LibraryCardElement): Promise<void> {
        if (!this.previewElement) {
            return;
        }

        if (!supportsHoverPreview()) {
            this.hideNow();
            return;
        }

        if (!card.isConnected) {
            return;
        }

        if (this.isUserScrolling) {
            return;
        }

        if (this.isVisible && this.targetCard === card) {
            return;
        }

        if (this.isVisible && this.targetCard && this.targetCard !== card) {
            this.fadeToCard(card);
            return;
        }

        this.targetCard = card;

        const data = await this.loadPreviewData(card);
        if (
            this.targetCard !== card ||
            !this.previewElement ||
            this.isUserScrolling
        ) {
            return;
        }

        if (!this.hasRenderableContent(data)) {
            this.hideNow();
            return;
        }

        this.render(data);
        this.setDescription(data.description);

        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';
        this.previewElement.style.visibility = 'hidden';
        this.previewElement.classList.remove('hidden');
        this.previewElement.setAttribute('aria-hidden', 'false');

        this.positionPreview(card, this.previewElement);

        requestAnimationFrame(() => {
            if (!this.previewElement) {
                return;
            }

            this.previewElement.style.visibility = 'visible';

            requestAnimationFrame(() => {
                if (!this.previewElement) {
                    return;
                }

                this.previewElement.style.opacity = '1';
                this.previewElement.style.transform = 'translateY(0) scale(1)';
                this.isVisible = true;
            });
        });
    }

    private render(data: PreviewCardData): void {
        if (!this.previewElement) {
            return;
        }

        const titleEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-title]',
        );
        const authorEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-author]',
        );
        const seriesEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-series]',
        );
        const descriptionEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-description]',
        );
        const descriptionWrapperEl =
            this.previewElement.querySelector<HTMLElement>(
                '[data-preview-description-wrapper]',
            );

        if (titleEl) {
            titleEl.textContent = data.title;
        }

        if (authorEl) {
            authorEl.textContent = data.author;
            authorEl.classList.toggle('hidden', !data.author);
        }

        if (seriesEl) {
            seriesEl.textContent = data.series;
            seriesEl.classList.toggle('hidden', !data.series);
        }

        if (descriptionEl) {
            descriptionEl.textContent = '';
        }

        if (descriptionWrapperEl) {
            descriptionWrapperEl.classList.add('hidden');
        }
    }

    private setDescription(rawDescription: string): void {
        if (!this.previewElement) {
            return;
        }

        const descriptionEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-description]',
        );
        const descriptionWrapperEl =
            this.previewElement.querySelector<HTMLElement>(
                '[data-preview-description-wrapper]',
            );

        if (!descriptionEl || !descriptionWrapperEl) {
            return;
        }

        descriptionEl.innerHTML = rawDescription;
        const hasText = (descriptionEl.textContent || '').trim().length > 0;
        descriptionWrapperEl.classList.toggle('hidden', !hasText);
    }

    private fallbackDataFromCard(card: LibraryCardElement): PreviewCardData {
        return {
            title: this.clean(card.dataset.libraryItemTitle),
            author: this.clean(card.dataset.libraryItemAuthors),
            series: this.clean(card.dataset.libraryItemSeries),
            description: '',
        };
    }

    private async loadPreviewData(
        card: LibraryCardElement,
    ): Promise<PreviewCardData> {
        const fallback = this.fallbackDataFromCard(card);

        const itemId = card.dataset.libraryItemId;
        const collection = card.dataset.libraryCollection as
            | LibraryCollection
            | undefined;

        if (!itemId || (collection !== 'books' && collection !== 'comics')) {
            return fallback;
        }

        const cacheKey = `${collection}:${itemId}`;
        if (this.detailsCache.has(cacheKey)) {
            return this.detailsCache.get(cacheKey)!;
        }

        try {
            const payload =
                collection === 'books'
                    ? await api.books.get<LibraryDetailPreviewResponse>(itemId)
                    : await api.comics.get<LibraryDetailPreviewResponse>(
                          itemId,
                      );

            if (!payload?.item) {
                return fallback;
            }

            const item = payload.item;

            const data: PreviewCardData = {
                title: this.clean(item.title) || fallback.title,
                author: this.formatAuthors(item.authors) || fallback.author,
                series: this.clean(item.series || '') || fallback.series,
                description: item.description || '',
            };

            this.detailsCache.set(cacheKey, data);
            return data;
        } catch {
            return fallback;
        }
    }

    private formatAuthors(authors: string[]): string {
        return this.clean(authors.filter(Boolean).join(', '));
    }

    private positionPreview(
        card: LibraryCardElement,
        preview: HTMLElement,
    ): void {
        const cardRect = card.getBoundingClientRect();

        preview.style.maxWidth = 'min(360px, calc(100vw - 20px))';
        preview.style.top = '0px';
        preview.style.left = '0px';

        const previousVisibility = preview.style.visibility;
        preview.style.visibility = 'hidden';
        preview.classList.remove('hidden');
        const previewRect = preview.getBoundingClientRect();
        preview.style.visibility = previousVisibility;

        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        const space = {
            top: cardRect.top,
            bottom: viewportHeight - cardRect.bottom,
            left: cardRect.left,
            right: viewportWidth - cardRect.right,
        };

        type Placement = 'top' | 'bottom' | 'left' | 'right';
        const neededHeight = previewRect.height + PREVIEW_OFFSET_PX;
        const neededWidth = previewRect.width + PREVIEW_OFFSET_PX;

        const placementScores: Array<{ placement: Placement; score: number }> =
            [
                {
                    placement: 'right',
                    score:
                        space.right >= neededWidth
                            ? 10000 + space.right
                            : space.right - neededWidth,
                },
                {
                    placement: 'left',
                    score:
                        space.left >= neededWidth
                            ? 10000 + space.left
                            : space.left - neededWidth,
                },
                {
                    placement: 'top',
                    score:
                        space.top >= neededHeight
                            ? 10000 + space.top
                            : space.top - neededHeight,
                },
                {
                    placement: 'bottom',
                    score:
                        space.bottom >= neededHeight
                            ? 10000 + space.bottom
                            : space.bottom - neededHeight,
                },
            ];

        placementScores.sort((left, right) => right.score - left.score);
        const placement = placementScores[0]?.placement ?? 'right';

        let left = 0;
        let top = 0;

        const cardCenterX = cardRect.left + cardRect.width / 2;
        const cardCenterY = cardRect.top + cardRect.height / 2;

        if (placement === 'right') {
            left = cardRect.right + PREVIEW_OFFSET_PX;
            top = cardCenterY - previewRect.height / 2;
        } else if (placement === 'left') {
            left = cardRect.left - previewRect.width - PREVIEW_OFFSET_PX;
            top = cardCenterY - previewRect.height / 2;
        } else if (placement === 'top') {
            left = cardCenterX - previewRect.width / 2;
            top = cardRect.top - previewRect.height - PREVIEW_OFFSET_PX;
        } else {
            left = cardCenterX - previewRect.width / 2;
            top = cardRect.bottom + PREVIEW_OFFSET_PX;
        }

        const maxLeft = Math.max(
            VIEWPORT_PADDING_PX,
            viewportWidth - previewRect.width - VIEWPORT_PADDING_PX,
        );
        const maxTop = Math.max(
            VIEWPORT_PADDING_PX,
            viewportHeight - previewRect.height - VIEWPORT_PADDING_PX,
        );

        left = Math.min(Math.max(left, VIEWPORT_PADDING_PX), maxLeft);
        top = Math.min(Math.max(top, VIEWPORT_PADDING_PX), maxTop);

        preview.style.top = `${top}px`;
        preview.style.left = `${left}px`;

        const arrow = preview.querySelector<HTMLElement>(
            '[data-preview-arrow]',
        );
        if (arrow) {
            const arrowClamp = 14;
            const arrowX = Math.min(
                Math.max(cardCenterX - left, arrowClamp),
                Math.max(arrowClamp, previewRect.width - arrowClamp),
            );
            const arrowY = Math.min(
                Math.max(cardCenterY - top, arrowClamp),
                Math.max(arrowClamp, previewRect.height - arrowClamp),
            );

            this.positionArrow(arrow, placement, arrowX, arrowY);
        }
    }

    private positionArrow(
        arrow: HTMLElement,
        placement: 'top' | 'bottom' | 'left' | 'right',
        arrowX: number,
        arrowY: number,
    ): void {
        arrow.style.top = '';
        arrow.style.bottom = '';
        arrow.style.left = '';
        arrow.style.right = '';

        if (placement === 'right') {
            arrow.style.left = '-6px';
            arrow.style.top = `${arrowY - 6}px`;
            arrow.style.transform = 'rotate(-45deg)';
            return;
        }

        if (placement === 'left') {
            arrow.style.right = '-6px';
            arrow.style.top = `${arrowY - 6}px`;
            arrow.style.transform = 'rotate(135deg)';
            return;
        }

        if (placement === 'top') {
            arrow.style.bottom = '-6px';
            arrow.style.left = `${arrowX - 6}px`;
            arrow.style.transform = 'rotate(225deg)';
            return;
        }

        arrow.style.top = '-6px';
        arrow.style.left = `${arrowX - 6}px`;
        arrow.style.transform = 'rotate(45deg)';
    }

    private ensurePreviewElement(): void {
        if (this.previewElement) {
            return;
        }

        const existingElement = document.getElementById(PREVIEW_ELEMENT_ID);
        if (existingElement) {
            this.previewElement = existingElement;
            return;
        }

        this.previewElement = document.createElement('aside');
        this.previewElement.id = PREVIEW_ELEMENT_ID;
        this.previewElement.className =
            'fixed z-[70] hidden w-[min(22rem,calc(100vw-1.25rem))] pointer-events-none rounded-3xl border border-gray-200/95 bg-white/95 p-4 shadow-[0_30px_70px_-28px_rgba(15,23,42,0.55)] ring-1 ring-black/5 backdrop-blur-sm will-change-transform dark:border-dark-600/80 dark:bg-dark-900/90 dark:ring-white/10';
        this.previewElement.setAttribute('role', 'tooltip');
        this.previewElement.setAttribute('aria-hidden', 'true');
        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';
        this.previewElement.style.visibility = 'hidden';
        this.previewElement.style.transition =
            'opacity 280ms cubic-bezier(0.22, 1, 0.36, 1), transform 280ms cubic-bezier(0.22, 1, 0.36, 1)';

        this.previewElement.innerHTML = `
            <span data-preview-arrow class="absolute h-3 w-3 rounded-sm border-l border-t border-gray-200/95 bg-white/95 dark:border-dark-600/80 dark:bg-dark-900/90"></span>
            <div data-preview-title class="text-[1rem] font-semibold leading-tight tracking-tight text-gray-900 dark:text-white"></div>
            <div data-preview-author class="mt-1 text-[11px] font-medium uppercase tracking-[0.05em] text-gray-500 dark:text-gray-300 hidden"></div>
            <div data-preview-series class="mt-2 w-fit rounded-full border border-gray-200 bg-gray-50 px-2 py-0.5 text-[10px] font-medium tracking-wide text-gray-600 dark:border-dark-500 dark:bg-dark-800 dark:text-gray-300 hidden"></div>
            <div data-preview-description-wrapper class="mt-3 border-t border-gray-200/80 pt-2.5 dark:border-dark-600/80 hidden">
                <div data-preview-description class="text-[13px] leading-relaxed text-gray-700/95 line-clamp-5 dark:text-gray-200"></div>
            </div>
        `;

        document.body.appendChild(this.previewElement);
    }

    private clearTimeouts(): void {
        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();
    }

    private clearShowTimeout(): void {
        if (this.showTimeoutId === null) {
            return;
        }

        window.clearTimeout(this.showTimeoutId);
        this.showTimeoutId = null;
    }

    private clearHideTimeout(): void {
        if (this.hideTimeoutId === null) {
            return;
        }

        window.clearTimeout(this.hideTimeoutId);
        this.hideTimeoutId = null;
    }

    private clearFadeTimeout(): void {
        if (this.fadeTimeoutId === null) {
            return;
        }

        window.clearTimeout(this.fadeTimeoutId);
        this.fadeTimeoutId = null;
    }

    private clearScrollIdleTimeout(): void {
        if (this.scrollIdleTimeoutId === null) {
            return;
        }

        window.clearTimeout(this.scrollIdleTimeoutId);
        this.scrollIdleTimeoutId = null;
    }

    private markScrolling(): void {
        this.isUserScrolling = true;
        this.clearShowTimeout();
        this.clearScrollIdleTimeout();

        this.scrollIdleTimeoutId = window.setTimeout(() => {
            this.isUserScrolling = false;
            this.scrollIdleTimeoutId = null;

            const card = this.hoveredCard;
            if (!card || !card.isConnected || this.isVisible) {
                return;
            }

            const isPointerOver = card.matches(':hover');
            const hasFocusWithin = card.contains(document.activeElement);
            if (isPointerOver || hasFocusWithin) {
                this.scheduleShow(card);
            }
        }, SCROLL_IDLE_MS);
    }

    private fadeToCard(card: LibraryCardElement): void {
        if (!this.previewElement) {
            return;
        }

        this.clearFadeTimeout();
        this.isVisible = false;
        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';

        this.fadeTimeoutId = window.setTimeout(() => {
            if (!this.previewElement) {
                return;
            }

            this.previewElement.style.visibility = 'hidden';
            this.previewElement.classList.add('hidden');
            this.previewElement.setAttribute('aria-hidden', 'true');
            void this.show(card);
        }, FADE_DURATION_MS);
    }

    private clean(value: string | undefined): string {
        return (value || '').replace(/\s+/g, ' ').trim();
    }

    private hasRenderableContent(data: PreviewCardData): boolean {
        return [data.title, data.author, data.series, data.description].some(
            (value) => this.clean(value).length > 0,
        );
    }
}

export function useLibraryHoverPreviewEffect(refreshKey: string): void {
    const detailsCacheRef = useRef<Map<string, PreviewCardData>>(new Map());

    useEffect(() => {
        const manager = new HoverPreviewManager(detailsCacheRef.current);
        return manager.init('.book-card');
    }, [refreshKey]);
}
