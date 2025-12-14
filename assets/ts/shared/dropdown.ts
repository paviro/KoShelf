// Global handler for closing details elements when clicking outside
document.addEventListener('click', (e: MouseEvent) => {
    const target = e.target instanceof Node ? e.target : null;
    if (!target) return;

    // Find all open details elements
    const openDetails = document.querySelectorAll<HTMLDetailsElement>('details[open]');

    openDetails.forEach((details) => {
        // If the click is not inside the details element, close it
        if (!details.contains(target)) {
            details.removeAttribute('open');
        }
    });

    // Handle generic dropdown menus (not using <details>)
    // This expects the menu to have .dropdown-menu or .dropdown-menu-right class
    // and rely on a 'hidden' class for visibility.
    // It also assumes the menu and trigger are siblings within a parent container.
    const genericDropdowns = document.querySelectorAll('.dropdown-menu, .dropdown-menu-right');
    genericDropdowns.forEach((menu) => {
        // Skip if inside a details element (handled above)
        if (menu.closest('details')) return;

        // Skip if already hidden
        if (menu.classList.contains('hidden')) return;

        // If click is outside the parent container (which likely includes trigger), close it
        // We use parentElement because normally the trigger is a sibling.
        if (menu.parentElement && !menu.parentElement.contains(target)) {
            menu.classList.add('hidden');

            // If a dropdown uses a rotated arrow icon, reset it when we close the menu.
            // Convention: any arrow that should be reset has class "dropdown-arrow".
            const arrows = menu.parentElement.querySelectorAll<HTMLElement>('.dropdown-arrow');
            arrows.forEach((arrow) => arrow.classList.remove('rotate-180'));
        }
    });
});
