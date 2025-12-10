// Section Toggle Module
// Handles collapsible sections with data-driven configuration

interface SectionData {
    section: HTMLElement;
    button: HTMLButtonElement;
    container: HTMLElement;
    chevron: SVGElement;
    buttonText: HTMLSpanElement;
    defaultVisible: boolean;
}

export class SectionToggle {
    private sections = new Map<string, SectionData>();

    constructor() {
        this.init();
    }

    private init(): void {
        // Find all sections with data-name attributes
        const toggleSections = document.querySelectorAll<HTMLElement>('section[data-name]');

        toggleSections.forEach(section => {
            const sectionName = section.dataset.name;
            if (!sectionName) return;

            const defaultVisible = section.dataset.defaultVisible === 'true';
            const button = section.querySelector('button');
            const container = section.querySelector<HTMLElement>('[id$="Container"]');
            const chevron = button?.querySelector('svg');
            const buttonText = button?.querySelector('span');

            if (button && container && chevron && buttonText) {
                // Store section references
                this.sections.set(sectionName, {
                    section,
                    button: button as HTMLButtonElement,
                    container,
                    chevron: chevron as SVGElement,
                    buttonText: buttonText as HTMLSpanElement,
                    defaultVisible
                });

                // Set initial state
                this.setInitialState(sectionName);

                // Add click event listener
                button.addEventListener('click', () => {
                    this.toggle(sectionName);
                });
            }
        });
    }

    private setInitialState(sectionName: string): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText, defaultVisible } = sectionData;

        if (defaultVisible) {
            // Show the section initially
            container.classList.remove('hidden');
            chevron.style.transform = 'rotate(0deg)';
            buttonText.textContent = 'Hide';
        } else {
            // Hide the section initially
            container.classList.add('hidden');
            chevron.style.transform = 'rotate(-90deg)';
            buttonText.textContent = 'Show';
        }
    }

    toggle(sectionName: string): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container } = sectionData;
        const isHidden = container.classList.contains('hidden');

        if (isHidden) {
            this.show(sectionName);
        } else {
            this.hide(sectionName);
        }
    }

    show(sectionName: string): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.remove('hidden');
        chevron.style.transform = 'rotate(0deg)';
        buttonText.textContent = 'Hide';
    }

    hide(sectionName: string): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.add('hidden');
        chevron.style.transform = 'rotate(-90deg)';
        buttonText.textContent = 'Show';
    }

    isVisible(sectionName: string): boolean {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return false;

        return !sectionData.container.classList.contains('hidden');
    }

    showAll(): void {
        this.sections.forEach((_, sectionName) => {
            this.show(sectionName);
        });
    }

    hideAll(): void {
        this.sections.forEach((_, sectionName) => {
            this.hide(sectionName);
        });
    }

    getSectionNames(): string[] {
        return Array.from(this.sections.keys());
    }
}
