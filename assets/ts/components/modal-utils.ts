// Shared modal animation utilities

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

/**
 * Setup standard modal close handlers (close button, backdrop click, Escape key)
 */
export function setupModalCloseHandlers(
    modal: HTMLElement | null,
    modalCard: HTMLElement | null,
    closeBtn: HTMLElement | null = null
): (() => void) | undefined {
    if (!modal || !modalCard) return;

    const hide = () => hideModal(modal, modalCard);

    // Close button
    if (closeBtn) {
        closeBtn.addEventListener('click', hide);
    }

    // Backdrop click
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            hide();
        }
    });

    // Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
            hide();
        }
    });

    return hide;
}
