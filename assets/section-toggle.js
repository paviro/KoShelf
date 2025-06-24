// Section Toggle Module
// Handles collapsible sections with data-driven configuration

class SectionToggle {
    constructor() {
        this.sections = new Map();
        this.init();
    }

    init() {
        // Find all sections with data-name attributes
        const toggleSections = document.querySelectorAll('section[data-name]');
        
        toggleSections.forEach(section => {
            const sectionName = section.dataset.name;
            const defaultVisible = section.dataset.defaultVisible === 'true';
            const button = section.querySelector('button');
            const container = section.querySelector('[id$="Container"]');
            const chevron = button?.querySelector('svg');
            const buttonText = button?.querySelector('span');
            
            if (button && container && chevron && buttonText) {
                // Store section references
                this.sections.set(sectionName, {
                    section,
                    button,
                    container,
                    chevron,
                    buttonText,
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

    setInitialState(sectionName) {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText, defaultVisible } = sectionData;

        if (defaultVisible) {
            // Show the section initially
            container.classList.remove('hidden');
            chevron.style.transform = 'rotate(180deg)';
            buttonText.textContent = 'Hide';
        } else {
            // Hide the section initially
            container.classList.add('hidden');
            chevron.style.transform = 'rotate(0deg)';
            buttonText.textContent = 'Show';
        }
    }

    toggle(sectionName) {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        const isHidden = container.classList.contains('hidden');

        if (isHidden) {
            this.show(sectionName);
        } else {
            this.hide(sectionName);
        }
    }

    show(sectionName) {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.remove('hidden');
        chevron.style.transform = 'rotate(180deg)';
        buttonText.textContent = 'Hide';
    }

    hide(sectionName) {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.add('hidden');
        chevron.style.transform = 'rotate(0deg)';
        buttonText.textContent = 'Show';
    }

    isVisible(sectionName) {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return false;

        return !sectionData.container.classList.contains('hidden');
    }

    showAll() {
        this.sections.forEach((_, sectionName) => {
            this.show(sectionName);
        });
    }

    hideAll() {
        this.sections.forEach((_, sectionName) => {
            this.hide(sectionName);
        });
    }

    getSectionNames() {
        return Array.from(this.sections.keys());
    }
}

// Export as ES6 module
export { SectionToggle }; 