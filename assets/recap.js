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
});
