// Section Toggle Module
// Handles collapsible sections with data-driven configuration

import { translation } from '../shared/i18n.js';
import { StorageManager } from '../shared/storage-manager.js';

interface SectionData {
    section: HTMLElement;
    button: HTMLButtonElement;
    container: HTMLElement;
    chevron: SVGElement;
    buttonText: HTMLSpanElement;
    defaultVisible: boolean;
}

type PersistedSectionState = Record<string, boolean>;

interface ToggleOptions {
    /**
     * Whether to persist the change (only applies when persistence is enabled for the page).
     * Defaults to true.
     */
    persist?: boolean;
}

export class SectionToggle {
    private sections = new Map<string, SectionData>();
    private persistenceKey: (typeof StorageManager.KEYS)[keyof typeof StorageManager.KEYS] | null = null;
    private persistedState: PersistedSectionState | null = null;

    constructor() {
        this.init();
    }

    private async init(): Promise<void> {
        // Load translations
        await translation.init();

        // Enable persistence only for pages that opt in (e.g. book/comic list)
        this.persistenceKey = this.resolvePersistenceKey();
        this.persistedState = this.persistenceKey
            ? (StorageManager.get<PersistedSectionState>(this.persistenceKey, {}) ?? {})
            : null;

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
                    this.toggle(sectionName, { persist: true });
                });
            }
        });
    }

    private resolvePersistenceKey():
        | (typeof StorageManager.KEYS)[keyof typeof StorageManager.KEYS]
        | null {
        const scope = document.body?.dataset.sectionToggleScope;
        const kind = document.body?.dataset.sectionToggleKind;

        // We cannot rely on the URL because comics may be served at "/" or "/comics/".
        // Templates provide a stable signal via data attributes.
        if (scope === 'library-list') {
            if (kind === 'comics') return StorageManager.KEYS.LIBRARY_LIST_COMICS_SECTIONS;
            if (kind === 'books') return StorageManager.KEYS.LIBRARY_LIST_BOOKS_SECTIONS;
            return null;
        }

        // Persist the same state across all detail pages (not per-book/per-comic).
        if (scope === 'item-details') {
            if (kind === 'comics') return StorageManager.KEYS.ITEM_DETAIL_COMICS_SECTIONS;
            if (kind === 'books') return StorageManager.KEYS.ITEM_DETAIL_BOOKS_SECTIONS;
            return null;
        }

        // Statistics can be scoped (all/books/comics); keep state separate per scope.
        if (scope === 'statistics') {
            if (kind === 'books') return StorageManager.KEYS.STATS_BOOKS_SECTIONS;
            if (kind === 'comics') return StorageManager.KEYS.STATS_COMICS_SECTIONS;
            if (kind === 'all') return StorageManager.KEYS.STATS_ALL_SECTIONS;
            return null;
        }

        return null;
    }

    private savePersistedState(): void {
        if (!this.persistenceKey || !this.persistedState) return;
        StorageManager.set(this.persistenceKey, this.persistedState);
    }

    private setInitialState(sectionName: string): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText, defaultVisible } = sectionData;
        const persistedVisible =
            this.persistedState && Object.prototype.hasOwnProperty.call(this.persistedState, sectionName)
                ? this.persistedState[sectionName]
                : null;
        const visible = persistedVisible === null ? defaultVisible : persistedVisible;

        if (visible) {
            // Show the section initially
            container.classList.remove('hidden');
            chevron.style.transform = 'rotate(0deg)';
            buttonText.textContent = translation.get('toggle.hide');
        } else {
            // Hide the section initially
            container.classList.add('hidden');
            chevron.style.transform = 'rotate(-90deg)';
            buttonText.textContent = translation.get('toggle.show');
        }
    }

    toggle(sectionName: string, options: ToggleOptions = {}): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container } = sectionData;
        const isHidden = container.classList.contains('hidden');

        if (isHidden) {
            this.show(sectionName, options);
        } else {
            this.hide(sectionName, options);
        }
    }

    show(sectionName: string, options: ToggleOptions = {}): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.remove('hidden');
        chevron.style.transform = 'rotate(0deg)';
        buttonText.textContent = translation.get('toggle.hide');

        const shouldPersist = options.persist ?? true;
        if (shouldPersist && this.persistedState) {
            this.persistedState[sectionName] = true;
            this.savePersistedState();
        }
    }

    hide(sectionName: string, options: ToggleOptions = {}): void {
        const sectionData = this.sections.get(sectionName);
        if (!sectionData) return;

        const { container, chevron, buttonText } = sectionData;
        container.classList.add('hidden');
        chevron.style.transform = 'rotate(-90deg)';
        buttonText.textContent = translation.get('toggle.show');

        const shouldPersist = options.persist ?? true;
        if (shouldPersist && this.persistedState) {
            this.persistedState[sectionName] = false;
            this.savePersistedState();
        }
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

    /**
     * Re-apply either the persisted visibility (if available) or the template default.
     * Useful after temporary UI changes (e.g. search auto-expansion).
     */
    restorePersistedOrDefault(): void {
        this.sections.forEach((sectionData, sectionName) => {
            const persistedVisible =
                this.persistedState && Object.prototype.hasOwnProperty.call(this.persistedState, sectionName)
                    ? this.persistedState[sectionName]
                    : null;
            const visible = persistedVisible === null ? sectionData.defaultVisible : persistedVisible;
            if (visible) {
                this.show(sectionName, { persist: false });
            } else {
                this.hide(sectionName, { persist: false });
            }
        });
    }
}
