function clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max);
}

export function scrollToHorizontalPosition(
    scrollContainer: HTMLElement | null,
    contentContainer: HTMLElement | null,
    targetPosition: number,
    viewportAnchorRatio = 0.5,
): void {
    if (!scrollContainer || !contentContainer) return;

    const containerWidth = scrollContainer.clientWidth;
    const contentWidth = contentContainer.scrollWidth;

    if (contentWidth <= containerWidth) return;

    const anchor = clamp(viewportAnchorRatio, 0, 1);
    const maxScroll = contentWidth - containerWidth;
    const desiredScroll = targetPosition - containerWidth * anchor;

    scrollContainer.scrollLeft = clamp(desiredScroll, 0, maxScroll);
}

export function scrollToHorizontalOverflowRatio(
    scrollContainer: HTMLElement | null,
    contentContainer: HTMLElement | null,
    overflowRatio: number,
): void {
    if (!scrollContainer || !contentContainer) return;

    const containerWidth = scrollContainer.clientWidth;
    const contentWidth = contentContainer.scrollWidth;

    if (contentWidth <= containerWidth) return;

    const maxScroll = contentWidth - containerWidth;
    scrollContainer.scrollLeft = maxScroll * clamp(overflowRatio, 0, 1);
}
