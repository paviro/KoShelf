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
});
