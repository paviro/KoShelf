interface OptionClassState {
    active: readonly string[];
    inactive: readonly string[];
}

export function setActiveOption(
    options: Iterable<HTMLElement>,
    selectedOption: HTMLElement | null,
    classState: OptionClassState,
): void {
    for (const option of options) {
        option.classList.remove(...classState.active);
        option.classList.add(...classState.inactive);
    }

    if (!selectedOption) return;

    selectedOption.classList.add(...classState.active);
    selectedOption.classList.remove(...classState.inactive);
}
