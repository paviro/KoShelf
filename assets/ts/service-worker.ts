// Service Worker for KoShelf PWA
// Implements manifest-based caching with differential updates

// Service Worker global scope type assertion
const sw = self as unknown as ServiceWorkerGlobalScope & typeof globalThis;

interface CacheManifest {
    version: string;
    files: Record<string, string>;
}

interface BroadcastMessage {
    type: string;
    error?: string;
    changedCount?: number;
}

const CACHE_NAME = 'koshelf-cache-v1';
const MANIFEST_URL = '/cache-manifest.json';
const BATCH_SIZE = 10;

const SKIP_CACHE_PATTERNS = [
    '/version.txt',
    '/api/events/version',
    '/service-worker.js',
    '/cache-manifest.json'
];

// =============================================================================
// Error Handling
// =============================================================================

sw.onerror = (event: Event | string) => {
    const message = typeof event === 'string' ? event : (event as ErrorEvent).message;
    console.error('[SW] Critical error:', message);
    broadcast({ type: 'CRITICAL_ERROR', error: String(message) });
    return false;
};

sw.onunhandledrejection = (event: PromiseRejectionEvent) => {
    console.error('[SW] Unhandled rejection:', event.reason);
    broadcast({ type: 'CRITICAL_ERROR', error: String(event.reason || 'Unhandled Rejection') });
};

// =============================================================================
// Client Communication
// =============================================================================

async function broadcast(message: BroadcastMessage): Promise<void> {
    const clients = await sw.clients.matchAll();
    clients.forEach((client) => client.postMessage(message));
}

// =============================================================================
// Cache Utilities
// =============================================================================

function shouldSkipCache(url: string): boolean {
    const pathname = new URL(url).pathname;
    return SKIP_CACHE_PATTERNS.some(pattern => pathname.endsWith(pattern));
}

function toFullUrl(urlPath: string): string {
    return new URL(urlPath, sw.location.origin).href;
}

// Normalize URL for cache matching - handles /foo/index.html -> /foo/ mapping
function normalizeUrlForCache(url: string): string {
    const parsed = new URL(url);
    let pathname = parsed.pathname;

    // Remove query string for cache lookup
    parsed.search = '';

    // If path ends with index.html, convert to directory form
    if (pathname.endsWith('/index.html')) {
        pathname = pathname.slice(0, -10); // Remove 'index.html', keep trailing '/'
        parsed.pathname = pathname;
    }

    return parsed.href;
}

async function cacheUrlsInBatches(cache: Cache, urlPaths: string[]): Promise<void> {
    for (let i = 0; i < urlPaths.length; i += BATCH_SIZE) {
        const batch = urlPaths.slice(i, i + BATCH_SIZE);
        await Promise.all(batch.map(urlPath => cacheUrl(cache, urlPath)));
    }
}

async function cacheUrl(cache: Cache, urlPath: string): Promise<void> {
    try {
        const fullUrl = toFullUrl(urlPath);
        const response = await fetch(fullUrl, { cache: 'no-store' });
        if (response.ok) {
            await cache.put(fullUrl, response.clone());
        } else {
            console.warn(`[SW] Failed to cache ${urlPath}: ${response.status}`);
        }
    } catch (err) {
        console.warn(`[SW] Failed to cache ${urlPath}:`, err);
    }
}

// =============================================================================
// Manifest Management
// =============================================================================

async function fetchManifest(): Promise<CacheManifest | null> {
    try {
        const response = await fetch(MANIFEST_URL, { cache: 'no-store' });
        return response.ok ? response.json() as Promise<CacheManifest> : null;
    } catch (e) {
        console.error('[SW] Failed to fetch manifest:', e);
        return null;
    }
}

async function getStoredManifest(): Promise<CacheManifest | null> {
    try {
        const cache = await caches.open(CACHE_NAME);
        const response = await cache.match(MANIFEST_URL);
        return response ? response.json() as Promise<CacheManifest> : null;
    } catch (e) {
        console.warn('[SW] Failed to get stored manifest:', e);
        return null;
    }
}

async function storeManifest(manifest: CacheManifest): Promise<void> {
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

// =============================================================================
// Cache Operations
// =============================================================================

async function precacheFiles(manifest: CacheManifest): Promise<void> {
    if (!manifest?.files) return;

    const cache = await caches.open(CACHE_NAME);
    const urlPaths = Object.keys(manifest.files);

    console.log(`[SW] Pre-caching ${urlPaths.length} files...`);
    await cacheUrlsInBatches(cache, urlPaths);
    console.log('[SW] Pre-caching complete');
}

async function updateChangedFiles(oldManifest: CacheManifest | null, newManifest: CacheManifest): Promise<void> {
    if (!newManifest?.files) return;

    const cache = await caches.open(CACHE_NAME);
    const oldFiles = oldManifest?.files || {};
    const newFiles = newManifest.files;

    const changedUrls = Object.keys(newFiles).filter(
        url => oldFiles[url] !== newFiles[url]
    );
    const deletedUrls = Object.keys(oldFiles).filter(
        url => !(url in newFiles)
    );

    console.log(`[SW] Updating: ${changedUrls.length} changed, ${deletedUrls.length} deleted`);

    // Remove deleted files
    await Promise.all(deletedUrls.map(url => cache.delete(toFullUrl(url))));

    // Cache changed files
    await cacheUrlsInBatches(cache, changedUrls);
    await storeManifest(newManifest);

    if (changedUrls.length > 0) {
        broadcast({ type: 'CACHE_UPDATED', changedCount: changedUrls.length });
    }

    console.log('[SW] Cache update complete');
}

// =============================================================================
// Event Handlers
// =============================================================================

sw.addEventListener('install', (event) => {
    console.log('[SW] Installing...');
    event.waitUntil((async () => {
        const manifest = await fetchManifest();
        if (manifest) {
            await precacheFiles(manifest);
            await storeManifest(manifest);
        }
        await sw.skipWaiting();
    })());
});

sw.addEventListener('activate', (event) => {
    console.log('[SW] Activating...');
    event.waitUntil((async () => {
        // Clean up old cache versions
        const cacheNames = await caches.keys();
        await Promise.all(
            cacheNames
                .filter(name => name !== CACHE_NAME)
                .map(name => caches.delete(name))
        );
        await sw.clients.claim();
    })());
});

sw.addEventListener('fetch', (event) => {
    const { request } = event;

    // Only handle GET requests for cacheable resources
    if (request.method !== 'GET' || shouldSkipCache(request.url)) {
        return;
    }

    event.respondWith(handleFetch(request));
});

async function handleFetch(request: Request): Promise<Response> {
    const cache = await caches.open(CACHE_NAME);

    // Normalize URL for cache matching (handles /foo/index.html -> /foo/ mapping)
    const normalizedUrl = normalizeUrlForCache(request.url);

    // Cache-first strategy - try normalized URL
    const cached = await cache.match(normalizedUrl, { ignoreVary: true });
    if (cached) return cached;

    // Network fallback with cache-busting
    try {
        const bustUrl = new URL(request.url);
        bustUrl.searchParams.set('_cb', String(Date.now()));

        const response = await fetch(bustUrl.toString(), {
            method: request.method,
            headers: request.headers,
            mode: 'cors',
            credentials: request.credentials
        });

        if (response.ok) {
            // Store with normalized URL for consistent cache keys
            cache.put(normalizedUrl, response.clone());
        }

        return response;
    } catch {
        // Offline - try index.html for navigation, else 503
        if (request.mode === 'navigate') {
            // Try root page - cache stores it as '/' not '/index.html'
            const index = await cache.match(toFullUrl('/'), { ignoreVary: true });
            if (index) return index;
        }

        return new Response('Offline', {
            status: 503,
            statusText: 'Service Unavailable'
        });
    }
}

interface MessageEventData {
    type: string;
}

sw.addEventListener('message', (event) => {
    const { type } = (event.data as MessageEventData) || {};

    const handlers: Record<string, () => void> = {
        SKIP_WAITING: () => sw.skipWaiting(),

        CLEAR_CACHE: () => event.waitUntil((async () => {
            await caches.delete(CACHE_NAME);
            broadcast({ type: 'CACHE_CLEARED' });
        })()),

        CHECK_FOR_UPDATES: () => event.waitUntil((async () => {
            console.log('[SW] Checking for updates...');
            const [oldManifest, newManifest] = await Promise.all([
                getStoredManifest(),
                fetchManifest()
            ]);

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
        })())
    };

    handlers[type]?.();
});
