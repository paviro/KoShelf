import { StorageManager } from './storage-manager.js';
import { showModal, setupModalCloseHandlers } from './modal-utils.js';
import { translation } from './i18n.js';

// Recap interactions: year dropdown + navigation
document.addEventListener('DOMContentLoaded', async () => {
    // Load translations
    await translation.init();

    const wrapper = document.getElementById('yearSelectorWrapper');
    const options = document.getElementById('yearOptions');
    const arrow = document.getElementById('yearDropdownArrow');

    if (wrapper && options) {
        // Toggle dropdown
        wrapper.addEventListener('click', (e) => {
            e.stopPropagation();
            options.classList.toggle('hidden');
            if (arrow) arrow.classList.toggle('rotate-180');
        });

        // Close on outside click
        document.addEventListener('click', (e) => {
            const target = e.target as Node;
            if (!wrapper.contains(target)) {
                options.classList.add('hidden');
                if (arrow) arrow.classList.remove('rotate-180');
            }
        });

        // Navigate when selecting a year
        document.querySelectorAll<HTMLElement>('.year-option').forEach((opt) => {
            opt.addEventListener('click', (e) => {
                e.stopPropagation();
                const y = opt.getAttribute('data-year');
                if (y) {
                    window.location.href = `/recap/${y}/`;
                }
            });
        });
    }

    // --- Sorting Logic ---
    const sortToggle = document.getElementById('sortToggle');
    const Timeline = document.getElementById('recapTimeline');

    if (sortToggle && Timeline) {
        // Read from storage, default to true (Newest First)
        let isNewestFirst = StorageManager.get<boolean>(StorageManager.KEYS.RECAP_SORT_ORDER, true) ?? true;

        // Icons
        // Newest First (Sort Descending): Lines + Arrow Down
        const iconNewest = `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h8M4 12h8M4 17h5M18 6v12M15 15l3 3 3-3"></path>`;

        // Oldest First (Sort Ascending): Lines + Arrow Up
        const iconOldest = `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h5M4 12h8M4 17h8M18 18V6M15 9l3-3 3 3"></path>`;

        // Function to update the button UI
        const updateUI = (): void => {
            const svg = sortToggle.querySelector('svg');
            if (svg) {
                svg.innerHTML = isNewestFirst ? iconNewest : iconOldest;
            }
            sortToggle.title = isNewestFirst ? translation.get('sort-order.newest-first') : translation.get('sort-order.oldest-first');
        };

        // Function to flip the DOM order
        const flipOrder = (): void => {
            // 1. Reorder Month Groups
            const months = Array.from(Timeline.querySelectorAll('.month-group'));
            months.reverse().forEach(month => Timeline.appendChild(month));

            // 2. Reorder Items within each Month Group (keep header at top)
            months.forEach(month => {
                const children = Array.from(month.children);
                if (children.length > 1) {
                    const header = children[0]; // first element is month title
                    const items = children.slice(1); // the rest are books

                    month.innerHTML = '';
                    month.appendChild(header);
                    items.reverse().forEach(item => month.appendChild(item));
                }
            });
        };

        // Apply initial state if different from default (Newest First)
        if (!isNewestFirst) {
            updateUI();
            flipOrder();
        }

        sortToggle.addEventListener('click', () => {
            isNewestFirst = !isNewestFirst;
            StorageManager.set(StorageManager.KEYS.RECAP_SORT_ORDER, isNewestFirst);

            updateUI();
            flipOrder();
        });
    }

    // --- Share Modal Logic ---
    const shareBtn = document.getElementById('shareButton');
    const shareModal = document.getElementById('shareModal');
    const shareModalCard = document.getElementById('shareModalCard');
    const shareModalClose = document.getElementById('shareModalClose');
    const shareModalTitle = document.getElementById('shareModalTitle');

    // Detect if we're on a mobile device (iOS, Android, etc.)
    const isMobileDevice = (): boolean => {
        return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) ||
            (navigator.maxTouchPoints !== undefined && navigator.maxTouchPoints > 2);
    };

    // Check if Web Share API is available and can share files
    const canUseWebShare = (): boolean => {
        return typeof navigator.share === 'function' && typeof navigator.canShare === 'function';
    };

    const isMobile = isMobileDevice();
    const useWebShare = isMobile && canUseWebShare();

    // Update button text and modal title based on device type
    if (useWebShare) {
        // Update modal title
        if (shareModalTitle) {
            shareModalTitle.textContent = translation.get('share.recap-label');
        }
        // Update button texts
        document.querySelectorAll('.share-btn-text').forEach(span => {
            span.textContent = translation.get('share');
        });
        // Update header button title/aria-label
        if (shareBtn) {
            shareBtn.title = translation.get('share.recap-label');
            shareBtn.setAttribute('aria-label', translation.get('share.recap-label'));
        }
    }

    if (shareBtn && shareModal && shareModalCard) {
        // Open modal with animation
        shareBtn.addEventListener('click', () => {
            showModal(shareModal, shareModalCard);
        });

        // Setup close handlers (X button, backdrop click, Escape key)
        setupModalCloseHandlers(shareModal, shareModalCard, shareModalClose);

        // Handle share/download button clicks
        document.querySelectorAll<HTMLElement>('.share-webp-btn').forEach(btn => {
            btn.addEventListener('click', async () => {
                const url = btn.dataset.shareUrl;
                const filename = btn.dataset.shareFilename;

                if (!url || !filename) return;

                if (useWebShare) {
                    // Use Web Share API on mobile
                    try {
                        // Fetch the image and convert to a File object
                        const response = await fetch(url);
                        const blob = await response.blob();
                        const file = new File([blob], filename, { type: 'image/webp' });

                        // Check if we can share this file
                        if (navigator.canShare && navigator.canShare({ files: [file] })) {
                            // Extract year from filename (e.g., "koshelf_2024_story.webp" -> "2024")
                            const yearMatch = filename.match(/koshelf_(\d{4})_/);
                            const year = yearMatch ? yearMatch[1] : String(new Date().getFullYear());

                            await navigator.share({
                                files: [file],
                                title: translation.get('my-reading-recap'),
                                text: `📚 My ${year} reading journey! These graphics were crafted by KoShelf, my KoReader reading companion. Check it out: https://github.com/paviro/KoShelf`,
                            });
                        } else {
                            // Fallback to download if file sharing isn't supported
                            triggerDownload(url, filename);
                        }
                    } catch (err) {
                        // User cancelled or error occurred - only log if it's not a user cancel
                        if (err instanceof Error && err.name !== 'AbortError') {
                            console.error('Share failed:', err);
                            // Fallback to download
                            triggerDownload(url, filename);
                        }
                    }
                } else {
                    // Use download on desktop
                    triggerDownload(url, filename);
                }
            });
        });
    }

    // Helper function to trigger a download
    function triggerDownload(url: string, filename: string): void {
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
    }
});
