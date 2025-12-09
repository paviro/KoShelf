// Service Worker for KoShelf PWA
// Implements manifest-based caching with differential updates

self.onerror = function (message, source, lineno, colno, error) {
    console.error('[SW] Critical error:', message, error);
    broadcastCriticalError(message);
    return false; // Let default handler run
};

self.onunhandledrejection = function (event) {
    console.error('[SW] Unhandled rejection:', event.reason);
    broadcastCriticalError(event.reason ? event.reason.toString() : 'Unhandled Rejection');
};

function broadcastCriticalError(errorMessage) {
    self.clients.matchAll().then(clients => {
        clients.forEach(client => {
            client.postMessage({
                type: 'CRITICAL_ERROR',
                error: errorMessage
            });
        });
    });
}


const CACHE_NAME = 'koshelf-cache-v1';
const MANIFEST_URL = '/cache-manifest.json';

// Files to skip caching (always fetch fresh)
const SKIP_CACHE_PATTERNS = [
    '/version.txt',
    '/server-mode.txt',
    '/version-poll',
    '/service-worker.js',
    '/cache-manifest.json'
];

function shouldSkipCache(url) {
    const pathname = new URL(url).pathname;
    return SKIP_CACHE_PATTERNS.some(pattern => pathname.endsWith(pattern));
}

async function fetchManifest() {
    try {
        const response = await fetch(MANIFEST_URL, { cache: 'no-store' });
        if (!response.ok) return null;
        return await response.json();
    } catch (e) {
        console.error('[SW] Failed to fetch manifest:', e);
        return null;
    }
}

async function getStoredManifest() {
    try {
        const cache = await caches.open(CACHE_NAME);
        const response = await cache.match(MANIFEST_URL);
        if (!response) return null;
        return await response.json();
    } catch (e) {
        console.warn('[SW] Failed to get stored manifest:', e);
        return null;
    }
}

async function storeManifest(manifest) {
    try {
        const cache = await caches.open(CACHE_NAME);
        const response = new Response(JSON.stringify(manifest), {
            headers: { 'Content-Type': 'application/json' }
        });
        await cache.put(MANIFEST_URL, response);
    } catch (e) {
        console.error('[SW] Failed to store manifest:', e);
    }
}

async function precacheFiles(manifest) {
    if (!manifest || !manifest.files) return;

    const cache = await caches.open(CACHE_NAME);
    const urls = Object.keys(manifest.files);

    console.log(`[SW] Pre-caching ${urls.length} files...`);

    // Cache files in parallel with a concurrency limit
    const BATCH_SIZE = 10;
    for (let i = 0; i < urls.length; i += BATCH_SIZE) {
        const batch = urls.slice(i, i + BATCH_SIZE);
        await Promise.all(batch.map(async (url) => {
            try {
                const response = await fetch(url, { cache: 'no-store' });
                if (response.ok) {
                    await cache.put(url, response);
                }
            } catch (e) {
                console.warn(`[SW] Failed to cache ${url}:`, e);
            }
        }));
    }

    console.log('[SW] Pre-caching complete');
}

async function updateChangedFiles(oldManifest, newManifest) {
    if (!newManifest || !newManifest.files) return;

    const cache = await caches.open(CACHE_NAME);
    const oldFiles = oldManifest?.files || {};
    const newFiles = newManifest.files;

    // Find changed files (new or different hash)
    const changedUrls = Object.keys(newFiles).filter(url => {
        return !oldFiles[url] || oldFiles[url] !== newFiles[url];
    });

    const deletedUrls = Object.keys(oldFiles).filter(url => !newFiles[url]);

    console.log(`[SW] Updating cache: ${changedUrls.length} changed, ${deletedUrls.length} deleted`);

    for (const url of deletedUrls) {
        await cache.delete(url);
    }

    // Fetch and cache changed files
    const BATCH_SIZE = 10;
    for (let i = 0; i < changedUrls.length; i += BATCH_SIZE) {
        const batch = changedUrls.slice(i, i + BATCH_SIZE);
        await Promise.all(batch.map(async (url) => {
            try {
                const response = await fetch(url, { cache: 'no-store' });
                if (response.ok) {
                    await cache.put(url, response);
                }
            } catch (e) {
                console.warn(`[SW] Failed to update ${url}:`, e);
            }
        }));
    }

    await storeManifest(newManifest);

    if (changedUrls.length > 0) {
        const clients = await self.clients.matchAll();
        clients.forEach(client => {
            client.postMessage({ type: 'CACHE_UPDATED', changedCount: changedUrls.length });
        });
    }

    console.log('[SW] Cache update complete');
}

// Install event - pre-cache all files from manifest
self.addEventListener('install', (event) => {
    console.log('[SW] Installing...');
    event.waitUntil(
        (async () => {
            const manifest = await fetchManifest();
            if (manifest) {
                await precacheFiles(manifest);
                await storeManifest(manifest);
            }
            // Skip waiting to activate immediately
            await self.skipWaiting();
        })()
    );
});

// Activate event - clean up old caches and take control
self.addEventListener('activate', (event) => {
    console.log('[SW] Activating...');
    event.waitUntil(
        (async () => {
            // Clean up old cache versions (but keep current ones)
            const cacheNames = await caches.keys();
            await Promise.all(
                cacheNames
                    .filter(name => name !== CACHE_NAME)
                    .map(name => caches.delete(name))
            );
            // Take control of all clients immediately
            await self.clients.claim();
        })()
    );
});

// Fetch event - cache-first strategy with network fallback
self.addEventListener('fetch', (event) => {
    const url = event.request.url;

    // Skip non-GET requests
    if (event.request.method !== 'GET') {
        return;
    }

    if (shouldSkipCache(url)) {
        return;
    }

    event.respondWith(
        (async () => {
            // Try cache first
            const cache = await caches.open(CACHE_NAME);
            const cachedResponse = await cache.match(event.request);

            if (cachedResponse) {
                return cachedResponse;
            }

            // Not in cache, fetch from network
            try {
                const networkResponse = await fetch(event.request);

                // Cache the response for future use
                if (networkResponse.ok) {
                    const responseToCache = networkResponse.clone();
                    cache.put(event.request, responseToCache);
                }

                return networkResponse;
            } catch (e) {
                // Network failed
                // If it's a navigation request, try to return cached index.html
                if (event.request.mode === 'navigate') {
                    const cachedIndex = await cache.match('/index.html');
                    if (cachedIndex) return cachedIndex;
                }

                // Return offline response
                return new Response('Offline', {
                    status: 503,
                    statusText: 'Service Unavailable'
                });
            }
        })()
    );
});

self.addEventListener('message', (event) => {
    if (event.data?.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }

    if (event.data?.type === 'CLEAR_CACHE') {
        event.waitUntil(
            (async () => {
                await caches.delete(CACHE_NAME);

                // Notify all clients that cache was cleared
                const clients = await self.clients.matchAll();
                clients.forEach(client => {
                    client.postMessage({ type: 'CACHE_CLEARED' });
                });
            })()
        );
    }

    if (event.data?.type === 'CHECK_FOR_UPDATES') {
        event.waitUntil(
            (async () => {
                console.log('[SW] Checking for updates...');
                const oldManifest = await getStoredManifest();
                const newManifest = await fetchManifest();

                if (!newManifest) {
                    console.log('[SW] Could not fetch new manifest');
                    return;
                }

                if (!oldManifest || oldManifest.version !== newManifest.version) {
                    console.log('[SW] New version detected, updating cache...');
                    await updateChangedFiles(oldManifest, newManifest);
                } else {
                    console.log('[SW] No updates needed');
                }
            })()
        );
    }
});
