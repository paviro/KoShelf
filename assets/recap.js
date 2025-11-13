// Minimal Recap UI interactions (year dropdown)
document.addEventListener('DOMContentLoaded', () => {
  const wrapper = document.getElementById('yearSelectorWrapper');
  const options = document.getElementById('yearOptions');
  const arrow = document.getElementById('dropdownArrow');
  if (!wrapper || !options) return;

  wrapper.addEventListener('click', (e) => {
    e.stopPropagation();
    options.classList.toggle('hidden');
    if (arrow) arrow.classList.toggle('rotate-180');
  });

  document.addEventListener('click', () => {
    options.classList.add('hidden');
    if (arrow) arrow.classList.remove('rotate-180');
  });
});


