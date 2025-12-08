import { StorageManager } from './storage-manager.js';

// Recap interactions: year dropdown + navigation
document.addEventListener('DOMContentLoaded', () => {
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
      if (!wrapper.contains(e.target)) {
        options.classList.add('hidden');
        if (arrow) arrow.classList.remove('rotate-180');
      }
    });

    // Navigate when selecting a year
    document.querySelectorAll('.year-option').forEach((opt) => {
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
    let isNewestFirst = StorageManager.get(StorageManager.KEYS.RECAP_SORT_ORDER, true);

    // Icons
    // Newest First (Sort Descending): Lines + Arrow Down
    // Lines: M4 7h8M4 12h8M4 17h5
    // Arrow: M18 6v12M15 15l3 3 3-3
    const iconNewest = `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h8M4 12h8M4 17h5M18 6v12M15 15l3 3 3-3"></path>`;

    // Oldest First (Sort Ascending): Lines + Arrow Up
    // Lines: M4 7h5M4 12h8M4 17h8
    // Arrow: M18 18V6M15 9l3-3 3 3
    const iconOldest = `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7h5M4 12h8M4 17h8M18 18V6M15 9l3-3 3 3"></path>`;

    // Function to update the button UI
    const updateUI = () => {
      const svg = sortToggle.querySelector('svg');
      if (svg) {
        svg.innerHTML = isNewestFirst ? iconNewest : iconOldest;
      }
      sortToggle.title = isNewestFirst ? "Current: Newest First" : "Current: Oldest First";
    };

    // Function to flip the DOM order
    const flipOrder = () => {
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
  const shareModalClose = document.getElementById('shareModalClose');
  const shareModalTitle = document.getElementById('shareModalTitle');

  // Detect if we're on a mobile device (iOS, Android, etc.)
  const isMobileDevice = () => {
    return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) ||
      (navigator.maxTouchPoints && navigator.maxTouchPoints > 2);
  };

  // Check if Web Share API is available and can share files
  const canUseWebShare = () => {
    return navigator.share && navigator.canShare;
  };

  const isMobile = isMobileDevice();
  const useWebShare = isMobile && canUseWebShare();

  // Update button text and modal title based on device type
  if (useWebShare) {
    // Update modal title
    if (shareModalTitle) {
      shareModalTitle.textContent = 'Share Recap Image';
    }
    // Update button texts
    document.querySelectorAll('.share-btn-text').forEach(span => {
      span.textContent = 'Share';
    });
    // Update header button title/aria-label
    if (shareBtn) {
      shareBtn.title = 'Share recap image';
      shareBtn.setAttribute('aria-label', 'Share recap image');
    }
  }

  if (shareBtn && shareModal) {
    // Open modal
    shareBtn.addEventListener('click', () => {
      shareModal.classList.remove('hidden');
      shareModal.classList.add('flex');
    });

    // Close modal on X button
    if (shareModalClose) {
      shareModalClose.addEventListener('click', () => {
        shareModal.classList.add('hidden');
        shareModal.classList.remove('flex');
      });
    }

    // Close modal on backdrop click
    shareModal.addEventListener('click', (e) => {
      if (e.target === shareModal) {
        shareModal.classList.add('hidden');
        shareModal.classList.remove('flex');
      }
    });

    // Close modal on Escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && !shareModal.classList.contains('hidden')) {
        shareModal.classList.add('hidden');
        shareModal.classList.remove('flex');
      }
    });

    // Handle share/download button clicks
    document.querySelectorAll('.share-png-btn').forEach(btn => {
      btn.addEventListener('click', async () => {
        const url = btn.dataset.shareUrl;
        const filename = btn.dataset.shareFilename;

        if (useWebShare) {
          // Use Web Share API on mobile
          try {
            // Fetch the image and convert to a File object
            const response = await fetch(url);
            const blob = await response.blob();
            const file = new File([blob], filename, { type: 'image/png' });

            // Check if we can share this file
            if (navigator.canShare({ files: [file] })) {
              // Extract year from filename (e.g., "koshelf_2024_story.png" -> "2024")
              const yearMatch = filename.match(/koshelf_(\d{4})_/);
              const year = yearMatch ? yearMatch[1] : new Date().getFullYear();

              await navigator.share({
                files: [file],
                title: 'My KoShelf Reading Recap',
                text: `📚 My ${year} reading journey! These graphics were crafted by KoShelf, my KoReader reading companion. Check it out: https://github.com/paviro/KoShelf`,
              });
            } else {
              // Fallback to download if file sharing isn't supported
              triggerDownload(url, filename);
            }
          } catch (err) {
            // User cancelled or error occurred - only log if it's not a user cancel
            if (err.name !== 'AbortError') {
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
  function triggerDownload(url, filename) {
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }
});
