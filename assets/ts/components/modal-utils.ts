// Shared modal animation utilities

type ModalRegistration = {
    modal: HTMLElement;
    modalCard: HTMLElement;
    hide: () => void;
    onBackdropClick: (e: MouseEvent) => void;
    closeBtn: HTMLElement | null;
};

const modalRegistrations = new Map<HTMLElement, ModalRegistration>();
let escapeHandlerInstalled = false;

/**
 * Show a modal with scale/fade animation
 */
export function showModal(modal: HTMLElement | null, modalCard: HTMLElement | null): void {
    if (!modal || !modalCard) return;

    modal.classList.remove('hidden');
    modal.classList.add('flex');
    modal.classList.add('opacity-0');
    modalCard.classList.add('scale-95', 'opacity-0');

    // Force reflow
    void modal.offsetHeight;

    requestAnimationFrame(() => {
        modal.classList.remove('opacity-0');
        modal.classList.add('opacity-100');
        modalCard.classList.remove('scale-95', 'opacity-0');
        modalCard.classList.add('scale-100', 'opacity-100');
    });
}

/**
 * Hide a modal with scale/fade animation
 */
export function hideModal(modal: HTMLElement | null, modalCard: HTMLElement | null): void {
    if (!modal || !modalCard) return;

    modal.classList.remove('opacity-100');
    modal.classList.add('opacity-0');
    modalCard.classList.remove('scale-100', 'opacity-100');
    modalCard.classList.add('scale-95', 'opacity-0');

    setTimeout(() => {
        modal.classList.add('hidden');
        modal.classList.remove('flex');
    }, 300);
}

function installEscapeHandlerOnce(): void {
    if (escapeHandlerInstalled) return;
    escapeHandlerInstalled = true;

    document.addEventListener('keydown', (e) => {
        if (e.key !== 'Escape') return;

        // Close any currently-visible registered modal.
        // Multiple modals can register handlers (e.g. calendar has 3),
        // but this keeps the global listener to a single instance.
        for (const { modal, hide } of modalRegistrations.values()) {
            if (!modal.classList.contains('hidden')) {
                hide();
            }
        }
    });
}

/**
 * Setup standard modal close handlers (close button, backdrop click, Escape key)
 */
export function setupModalCloseHandlers(
    modal: HTMLElement | null,
    modalCard: HTMLElement | null,
    closeBtn: HTMLElement | null = null,
): (() => void) | undefined {
    if (!modal || !modalCard) return;

    // Avoid stacking handlers if called multiple times for the same modal.
    const existing = modalRegistrations.get(modal);
    if (existing) return existing.hide;

    const hide = (): void => hideModal(modal, modalCard);

    // Backdrop click
    const onBackdropClick = (e: MouseEvent): void => {
        if (e.target === modal) hide();
    };
    modal.addEventListener('click', onBackdropClick);

    // Close button
    closeBtn?.addEventListener('click', hide);

    modalRegistrations.set(modal, { modal, modalCard, hide, onBackdropClick, closeBtn });
    installEscapeHandlerOnce();

    return hide;
}
