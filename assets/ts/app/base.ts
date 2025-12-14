// Base bundle loaded on all pages.
// Keep this file small: only global, page-agnostic behavior should live here.

import '../shared/pwa.js';
import '../shared/dropdown.js';
import '../shared/filter-restore.js';

// Side-effect import: runs on WebKit only.
import '../components/webkit-repaint-hack.js';
