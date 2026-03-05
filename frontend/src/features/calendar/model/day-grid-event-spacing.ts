const MARGIN_EPSILON_PX = 0.5;

function parsePixelValue(value: string | undefined): number | null {
    if (!value) {
        return null;
    }

    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : null;
}

function resolveGridRow(eventElement: HTMLElement): number {
    const rowFromGridRowStart = Number.parseInt(
        eventElement.style.gridRowStart,
        10,
    );
    if (Number.isFinite(rowFromGridRowStart)) {
        return rowFromGridRowStart;
    }

    const rowToken = eventElement.style.gridArea.split('/')[0]?.trim();
    const rowFromGridArea = Number.parseInt(rowToken ?? '', 10);
    return Number.isFinite(rowFromGridArea) ? rowFromGridArea : 1;
}

function approximatelyEqual(left: number, right: number): boolean {
    return Math.abs(left - right) <= MARGIN_EPSILON_PX;
}

function resolveBaseMargin(
    eventElement: HTMLElement,
    currentMargin: number,
): number {
    const storedBaseMargin = parsePixelValue(
        eventElement.dataset.baseMarginBlockStart,
    );
    const storedAdjustedMargin = parsePixelValue(
        eventElement.dataset.adjustedMarginBlockStart,
    );

    if (
        storedBaseMargin !== null &&
        storedAdjustedMargin !== null &&
        approximatelyEqual(currentMargin, storedAdjustedMargin)
    ) {
        return storedBaseMargin;
    }

    return currentMargin;
}

export function applyDayGridStackedEventGap(
    calendarContainer: HTMLElement,
    stackedEventVerticalGapPx: number,
): void {
    const eventElements = Array.from(
        calendarContainer.querySelectorAll<HTMLElement>(
            '.ec.ec-day-grid .ec-events > .ec-event',
        ),
    );

    if (eventElements.length === 0) {
        return;
    }

    const marginLevelsByGridRow = new Map<number, number[]>();

    for (const eventElement of eventElements) {
        const currentMargin = parsePixelValue(
            eventElement.style.marginBlockStart,
        );
        if (currentMargin === null) {
            continue;
        }

        const baseMargin = resolveBaseMargin(eventElement, currentMargin);
        eventElement.dataset.baseMarginBlockStart = `${baseMargin}`;

        const gridRow = resolveGridRow(eventElement);
        const levels = marginLevelsByGridRow.get(gridRow) ?? [];
        if (
            !levels.some((candidateMargin) =>
                approximatelyEqual(candidateMargin, baseMargin),
            )
        ) {
            levels.push(baseMargin);
            levels.sort((a, b) => a - b);
        }
        marginLevelsByGridRow.set(gridRow, levels);
    }

    for (const eventElement of eventElements) {
        const baseMargin = parsePixelValue(
            eventElement.dataset.baseMarginBlockStart,
        );
        if (baseMargin === null) {
            continue;
        }

        const gridRow = resolveGridRow(eventElement);
        const levels = marginLevelsByGridRow.get(gridRow) ?? [];
        const levelIndex = levels.findIndex((candidateMargin) =>
            approximatelyEqual(candidateMargin, baseMargin),
        );
        const adjustedMargin =
            levelIndex <= 0 || stackedEventVerticalGapPx <= 0
                ? baseMargin
                : baseMargin + levelIndex * stackedEventVerticalGapPx;

        eventElement.style.marginBlockStart = `${adjustedMargin}px`;
        eventElement.dataset.adjustedMarginBlockStart = `${adjustedMargin}`;
    }
}
