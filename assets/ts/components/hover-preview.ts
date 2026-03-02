interface PreviewCardData {
    title: string;
    author: string;
    series: string;
    description: string;
}

interface DetailsPayloadItem {
    title?: string | null;
    authors?: string[] | string | null;
    series?: string | null;
    description?: string | null;
}

interface DetailsPayload {
    book?: DetailsPayloadItem;
    comic?: DetailsPayloadItem;
}

const SHOW_DELAY_MS = 160;
const HIDE_DELAY_MS = 90;
const FADE_DURATION_MS = 200;
const SCROLL_IDLE_MS = 140;
const PREVIEW_OFFSET_PX = 14;
const VIEWPORT_PADDING_PX = 10;

class HoverPreviewManager {
    private previewElement: HTMLElement | null = null;
    private targetCard: HTMLElement | null = null;
    private isVisible = false;
    private showTimeoutId: number | null = null;
    private hideTimeoutId: number | null = null;
    private fadeTimeoutId: number | null = null;
    private scrollIdleTimeoutId: number | null = null;
    private isUserScrolling = false;
    private hoveredCard: HTMLElement | null = null;
    private detailsCache = new Map<string, PreviewCardData>();

    init(selector: string): void {
        if (!window.matchMedia('(pointer: fine)').matches) return;
        if (!window.matchMedia('(hover: hover)').matches) return;
        if (window.matchMedia('(max-width: 1023px)').matches) return;

        this.ensurePreviewElement();

        const cards = document.querySelectorAll<HTMLElement>(selector);
        cards.forEach((card) => {
            card.addEventListener('mouseenter', () => {
                this.hoveredCard = card;
                void this.loadPreviewData(card);
                this.scheduleShow(card);
            });
            card.addEventListener('mouseleave', () => {
                if (this.hoveredCard === card) {
                    this.hoveredCard = null;
                }
                this.scheduleHide();
            });
            card.addEventListener('focusin', () => {
                this.hoveredCard = card;
                void this.loadPreviewData(card);
                this.scheduleShow(card);
            });
            card.addEventListener('focusout', () => {
                if (this.hoveredCard === card && !card.matches(':hover')) {
                    this.hoveredCard = null;
                }
                this.scheduleHide();
            });
        });

        window.addEventListener(
            'scroll',
            () => {
                this.markScrolling();
                if (!this.isVisible) return;
                this.hideWithAnimation();
            },
            { passive: true, capture: true },
        );

        window.addEventListener(
            'resize',
            () => {
                if (!this.previewElement || !this.targetCard || !this.isVisible) return;
                this.positionPreview(this.targetCard, this.previewElement);
            },
            { passive: true },
        );

        document.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') {
                this.hideWithAnimation();
            }
        });
    }

    hideNow(): void {
        this.clearTimeouts();
        this.clearScrollIdleTimeout();
        this.hoveredCard = null;
        if (!this.previewElement) return;

        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';
        this.previewElement.style.visibility = 'hidden';
        this.previewElement.classList.add('hidden');

        this.targetCard = null;
        this.isVisible = false;
    }

    private scheduleShow(card: HTMLElement): void {
        if (this.isUserScrolling) return;
        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();

        this.showTimeoutId = window.setTimeout(() => {
            this.show(card);
        }, SHOW_DELAY_MS);
    }

    private scheduleHide(): void {
        this.clearShowTimeout();

        this.hideTimeoutId = window.setTimeout(() => {
            this.hideWithAnimation();
        }, HIDE_DELAY_MS);
    }

    private hideWithAnimation(): void {
        if (!this.previewElement) return;

        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();

        this.isVisible = false;
        this.targetCard = null;

        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';

        this.fadeTimeoutId = window.setTimeout(() => {
            if (!this.previewElement) return;
            this.previewElement.style.visibility = 'hidden';
            this.previewElement.classList.add('hidden');
            this.fadeTimeoutId = null;
        }, FADE_DURATION_MS);
    }

    private async show(card: HTMLElement): Promise<void> {
        if (!this.previewElement) return;
        if (this.isUserScrolling) return;
        if (this.isVisible && this.targetCard === card) return;

        if (this.isVisible && this.targetCard && this.targetCard !== card) {
            this.fadeToCard(card);
            return;
        }

        this.targetCard = card;

        const data = await this.loadPreviewData(card);
        if (this.targetCard !== card || !this.previewElement || this.isUserScrolling) return;
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
        this.positionPreview(card, this.previewElement);

        requestAnimationFrame(() => {
            if (!this.previewElement) return;
            this.previewElement.style.visibility = 'visible';

            requestAnimationFrame(() => {
                if (!this.previewElement) return;
                this.previewElement.style.opacity = '1';
                this.previewElement.style.transform = 'translateY(0) scale(1)';
                this.isVisible = true;
            });
        });
    }

    private render(data: PreviewCardData): void {
        if (!this.previewElement) return;

        const titleEl = this.previewElement.querySelector<HTMLElement>('[data-preview-title]');
        const authorEl = this.previewElement.querySelector<HTMLElement>('[data-preview-author]');
        const seriesEl = this.previewElement.querySelector<HTMLElement>('[data-preview-series]');
        const descriptionEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-description]',
        );
        if (titleEl) titleEl.textContent = data.title;
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
            descriptionEl.classList.add('hidden');
        }
    }

    private setDescription(rawDescription: string): void {
        if (!this.previewElement) return;
        const descriptionEl = this.previewElement.querySelector<HTMLElement>(
            '[data-preview-description]',
        );
        if (!descriptionEl) return;

        descriptionEl.innerHTML = rawDescription;
        const hasText = (descriptionEl.textContent || '').trim().length > 0;
        descriptionEl.classList.toggle('hidden', !hasText);
    }

    private async loadPreviewData(card: HTMLElement): Promise<PreviewCardData> {
        const detailsUrl = this.getDetailsUrl(card);
        if (!detailsUrl) {
            return {
                title: '',
                author: '',
                series: '',
                description: '',
            };
        }

        const cacheKey = detailsUrl.toString();
        if (this.detailsCache.has(cacheKey)) {
            return (
                this.detailsCache.get(cacheKey) || {
                    title: '',
                    author: '',
                    series: '',
                    description: '',
                }
            );
        }

        try {
            const response = await fetch(detailsUrl.toString(), {
                method: 'GET',
                headers: { Accept: 'application/json' },
            });
            if (!response.ok) {
                return {
                    title: '',
                    author: '',
                    series: '',
                    description: '',
                };
            }

            const payload = (await response.json()) as DetailsPayload;
            const item = payload.book || payload.comic;

            const data: PreviewCardData = {
                title: this.clean(item?.title || ''),
                author: this.formatAuthors(item?.authors),
                series: this.clean(item?.series || ''),
                description: typeof item?.description === 'string' ? item.description : '',
            };

            this.detailsCache.set(cacheKey, data);
            return data;
        } catch {
            return {
                title: '',
                author: '',
                series: '',
                description: '',
            };
        }
    }

    private getDetailsUrl(card: HTMLElement): URL | null {
        const link = card.querySelector<HTMLAnchorElement>('a[href]');
        const href = link?.getAttribute('href');
        if (!href) return null;
        return new URL('details.json', new URL(href, window.location.origin));
    }

    private formatAuthors(authors: string[] | string | null | undefined): string {
        if (Array.isArray(authors)) {
            return this.clean(authors.filter(Boolean).join(', '));
        }
        return this.clean(authors || '');
    }

    private positionPreview(card: HTMLElement, preview: HTMLElement): void {
        const cardRect = card.getBoundingClientRect();

        preview.style.maxWidth = 'min(360px, calc(100vw - 20px))';
        preview.style.top = '0px';
        preview.style.left = '0px';

        const prevVisibility = preview.style.visibility;
        preview.style.visibility = 'hidden';
        preview.classList.remove('hidden');
        const previewRect = preview.getBoundingClientRect();
        preview.style.visibility = prevVisibility;

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

        const placementScores: Array<{ placement: Placement; score: number }> = [
            {
                placement: 'right',
                score: space.right >= neededWidth ? 10000 + space.right : space.right - neededWidth,
            },
            {
                placement: 'left',
                score: space.left >= neededWidth ? 10000 + space.left : space.left - neededWidth,
            },
            {
                placement: 'top',
                score: space.top >= neededHeight ? 10000 + space.top : space.top - neededHeight,
            },
            {
                placement: 'bottom',
                score:
                    space.bottom >= neededHeight
                        ? 10000 + space.bottom
                        : space.bottom - neededHeight,
            },
        ];

        placementScores.sort((a, b) => b.score - a.score);
        const placement = placementScores[0]?.placement ?? 'right';

        let left: number;
        let top: number;

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

        const arrow = preview.querySelector<HTMLElement>('[data-preview-arrow]');
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
        if (this.previewElement) return;

        this.previewElement = document.createElement('aside');
        this.previewElement.className =
            'fixed z-[70] hidden w-[min(22rem,calc(100vw-1.25rem))] pointer-events-none rounded-3xl border border-gray-200/95 bg-white/95 p-4 shadow-[0_30px_70px_-28px_rgba(15,23,42,0.55)] ring-1 ring-black/5 backdrop-blur-sm will-change-transform dark:border-dark-600/80 dark:bg-dark-900/90 dark:ring-white/10';
        this.previewElement.setAttribute('role', 'tooltip');
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
            <div data-preview-description class="mt-3 border-t border-gray-200/80 pt-2.5 text-[13px] leading-relaxed text-gray-700/95 line-clamp-5 dark:border-dark-600/80 dark:text-gray-200 hidden"></div>
        `;

        document.body.appendChild(this.previewElement);
    }

    private clearTimeouts(): void {
        this.clearShowTimeout();
        this.clearHideTimeout();
        this.clearFadeTimeout();
    }

    private clearShowTimeout(): void {
        if (this.showTimeoutId === null) return;
        window.clearTimeout(this.showTimeoutId);
        this.showTimeoutId = null;
    }

    private clearHideTimeout(): void {
        if (this.hideTimeoutId === null) return;
        window.clearTimeout(this.hideTimeoutId);
        this.hideTimeoutId = null;
    }

    private clearFadeTimeout(): void {
        if (this.fadeTimeoutId === null) return;
        window.clearTimeout(this.fadeTimeoutId);
        this.fadeTimeoutId = null;
    }

    private clearScrollIdleTimeout(): void {
        if (this.scrollIdleTimeoutId === null) return;
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
            if (!card || !card.isConnected || this.isVisible) return;

            const isPointerOver = card.matches(':hover');
            const hasFocusWithin = card.contains(document.activeElement);
            if (isPointerOver || hasFocusWithin) {
                this.scheduleShow(card);
            }
        }, SCROLL_IDLE_MS);
    }

    private fadeToCard(card: HTMLElement): void {
        if (!this.previewElement) return;

        this.clearFadeTimeout();
        this.isVisible = false;
        this.previewElement.style.opacity = '0';
        this.previewElement.style.transform = 'translateY(6px) scale(0.98)';

        this.fadeTimeoutId = window.setTimeout(() => {
            if (!this.previewElement) return;
            this.previewElement.style.visibility = 'hidden';
            this.previewElement.classList.add('hidden');
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

const hoverPreviewManager = new HoverPreviewManager();

export function initBookCardHoverPreview(): void {
    hoverPreviewManager.init('.book-card');
}

export function hideBookCardHoverPreview(): void {
    hoverPreviewManager.hideNow();
}
