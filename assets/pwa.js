// PWA utilities - service worker registration and update notifications
import { StorageManager } from './storage-manager.js';

if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/service-worker.js').catch(error => {
        console.error('Service Worker registration failed:', error);
        recoveryReload();
    });

    navigator.serviceWorker.addEventListener('message', (e) => {
        if (e.data?.type === 'CACHE_UPDATED') {
            showUpdateToast();
        } else if (e.data?.type === 'CRITICAL_ERROR') {
            console.error('Service Worker reported critical error:', e.data.error);
            recoveryReload();
        }
    });

    // Global error handlers for main thread
    window.addEventListener('error', (event) => {
        // Broadly catch errors to recover from broken deployments (e.g. syntax errors, missing files)
        // We rely on recoveryReload()'s loop protection to prevent infinite reloads for persistent bugs
        if (event.error || event.message) {
            console.error('Recoverable window error detected:', event.error || event.message);
            recoveryReload();
        }
    });

    // Recovery mechanism - auto-reload if things are broken
    function recoveryReload() {
        // Prevent infinite reload loops
        let count = parseInt(StorageManager.get(StorageManager.KEYS.RELOAD_COUNT) || '0');
        const lastReload = parseInt(StorageManager.get(StorageManager.KEYS.LAST_RELOAD) || '0');
        const now = Date.now();

        // Reset count if last reload was more than 1 minute ago
        if (now - lastReload > 60000) {
            count = 0;
        }

        if (count >= 3) {
            console.error('Too many reloads, stopping auto-recovery to prevent loop.');
            return;
        }

        StorageManager.set(StorageManager.KEYS.RELOAD_COUNT, (count + 1).toString());
        StorageManager.set(StorageManager.KEYS.LAST_RELOAD, now.toString());

        console.log('Attempting recovery reload...');

        navigator.serviceWorker.getRegistrations().then(registrations => {
            for (let registration of registrations) {
                registration.unregister();
            }

            // Force reload ignoring cache
            window.location.reload(true);
        });
    }

    // Version checking - use long-polling if available, fallback to regular polling
    // Use StorageManager to track the version we loaded with
    let initialVersion = StorageManager.get(StorageManager.KEYS.VERSION);
    let hasShownToast = false;

    function handleVersionChange(version) {
        if (initialVersion === null) {
            // First check on this page load - store the version
            initialVersion = version;
            StorageManager.set(StorageManager.KEYS.VERSION, version);
        } else if (initialVersion !== version && !hasShownToast) {
            hasShownToast = true;

            StorageManager.set(StorageManager.KEYS.VERSION, version);

            if (navigator.serviceWorker.controller) {
                navigator.serviceWorker.controller.postMessage({ type: 'CHECK_FOR_UPDATES' });
            } else {
                showUpdateToast();
            }
        }
    }

    // Long-polling - keeps connection open until version changes or timeout (60s)
    async function longPoll() {
        while (true) {
            try {
                const response = await fetch('/version-poll', { cache: 'no-store' });

                if (response.status === 200) {
                    const version = (await response.text()).trim();
                    handleVersionChange(version);
                } else if (response.status === 204) {
                    // Timeout, reconnect immediately
                    continue;
                }
            } catch (e) {
                // Connection error, wait a bit and retry
                await new Promise(resolve => setTimeout(resolve, 5000));
            }
        }
    }

    let pollingInterval = null;

    async function checkVersion() {
        try {
            const response = await fetch('/version.txt', { cache: 'no-store' });
            if (!response.ok) return;

            const version = (await response.text()).trim();
            handleVersionChange(version);
        } catch (e) {
            // Offline or error, ignore
        }
    }

    function startRegularPolling() {
        if (pollingInterval) return; // Already polling
        checkVersion();
        pollingInterval = setInterval(checkVersion, 10000);
    }

    // Check if server supports long-polling (cached in StorageManager)
    async function checkServerMode() {
        const cachedMode = StorageManager.get(StorageManager.KEYS.SERVER_MODE);
        if (cachedMode !== null) {
            return cachedMode === 'internal';
        }

        // Check server-mode.txt - returns "internal" or "external"
        try {
            const response = await fetch('/server-mode.txt', { cache: 'no-store' });
            if (response.ok) {
                const mode = (await response.text()).trim();
                StorageManager.set(StorageManager.KEYS.SERVER_MODE, mode);
                return mode === 'internal';
            }
        } catch (e) {
            // Error fetching, assume external
        }

        StorageManager.set(StorageManager.KEYS.SERVER_MODE, 'external');
        return false;
    }

    // Start after a short delay to let the page fully load
    setTimeout(async () => {
        // First, get initial version via regular request
        try {
            const response = await fetch('/version.txt', { cache: 'no-store' });
            if (response.ok) {
                const version = (await response.text()).trim();
                handleVersionChange(version);
            }
        } catch (e) {
            // Ignore errors
        }

        const supportsLongPolling = await checkServerMode();
        if (supportsLongPolling) {
            longPoll();
        } else {
            startRegularPolling();
        }
    }, 1000);

    function showUpdateToast() {
        document.body.classList.add('update-available');
    }
}
