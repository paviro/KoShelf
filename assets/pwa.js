// PWA utilities - service worker registration and update notifications
import { StorageManager } from './storage-manager.js';

const KEYS = StorageManager.KEYS;
const RECOVERY_CONFIG = { maxRetries: 3, cooldownMs: 60000 };
const RECONNECT_DELAY_MS = 5000;
const POLL_INTERVAL_MS = 10000;
const SERVER_MODE = '{{SERVER_MODE}}'; // Injected at build time

// Version tracking state
let initialVersion = StorageManager.get(KEYS.VERSION);
let lastNotifiedVersion = null; // Track which version we last triggered an update for

if ('serviceWorker' in navigator) {
    initializeServiceWorker();
}

async function initializeServiceWorker() {
    try {
        await navigator.serviceWorker.register('/service-worker.js');
    } catch (error) {
        console.error('[PWA] Service Worker registration failed:', error);
        recoveryReload();
    }

    navigator.serviceWorker.addEventListener('message', handleServiceWorkerMessage);
    window.addEventListener('error', handleWindowError);

    // Start version monitoring after page load
    setTimeout(startVersionMonitoring, 1000);
}

function handleServiceWorkerMessage({ data }) {
    if (data?.type === 'CACHE_UPDATED') {
        showUpdateNotification();
    } else if (data?.type === 'CRITICAL_ERROR') {
        console.error('[PWA] Service Worker critical error:', data.error);
        recoveryReload();
    }
}

function handleWindowError(event) {
    if (event.error || event.message) {
        console.error('[PWA] Window error detected:', event.error || event.message);
        recoveryReload();
    }
}

function recoveryReload() {
    const now = Date.now();
    const lastReload = parseInt(StorageManager.get(KEYS.LAST_RELOAD) || '0');
    let count = parseInt(StorageManager.get(KEYS.RELOAD_COUNT) || '0');

    // Reset counter after cooldown period
    if (now - lastReload > RECOVERY_CONFIG.cooldownMs) {
        count = 0;
    }

    if (count >= RECOVERY_CONFIG.maxRetries) {
        console.error('[PWA] Recovery limit reached, stopping auto-reload');
        return;
    }

    StorageManager.set(KEYS.RELOAD_COUNT, String(count + 1));
    StorageManager.set(KEYS.LAST_RELOAD, String(now));

    console.log('[PWA] Attempting recovery reload...');

    navigator.serviceWorker.getRegistrations()
        .then(regs => regs.forEach(r => r.unregister()))
        .then(() => location.reload());
}

// Version monitoring
async function startVersionMonitoring() {
    const version = await fetchVersion();
    if (version) handleVersionChange(version);

    if (SERVER_MODE === 'internal') {
        startLongPolling();
    } else {
        startIntervalPolling();
    }
}

async function fetchVersion() {
    try {
        const response = await fetch('/version.txt', { cache: 'no-store' });
        return response.ok ? (await response.text()).trim() : null;
    } catch {
        return null;
    }
}

function handleVersionChange(version) {
    console.log(`[PWA] Version check: stored=${initialVersion}, fetched=${version}, lastNotified=${lastNotifiedVersion}`);

    if (initialVersion === null) {
        initialVersion = version;
        StorageManager.set(KEYS.VERSION, version);
        console.log('[PWA] First load, storing initial version');
        return;
    }

    // Trigger update if version changed from initial AND we haven't already notified for this specific version
    if (initialVersion !== version && lastNotifiedVersion !== version) {
        console.log('[PWA] Version changed! Requesting cache update...');
        lastNotifiedVersion = version;
        StorageManager.set(KEYS.VERSION, version);
        requestCacheUpdate();
    }
}

function requestCacheUpdate() {
    if (navigator.serviceWorker.controller) {
        console.log('[PWA] Sending CHECK_FOR_UPDATES to service worker');
        navigator.serviceWorker.controller.postMessage({ type: 'CHECK_FOR_UPDATES' });
    } else {
        console.log('[PWA] No SW controller, showing notification directly');
        showUpdateNotification();
    }
}

async function startLongPolling() {
    while (true) {
        try {
            const response = await fetch('/api/events/version', { cache: 'no-store' });

            if (response.status === 200) {
                handleVersionChange((await response.text()).trim());
            }
            // 204 = timeout, loop continues automatically
        } catch {
            // Connection lost - wait for server to come back
            await waitForServerRecovery();
        }
    }
}

async function waitForServerRecovery() {
    while (true) {
        await sleep(RECONNECT_DELAY_MS);
        const version = await fetchVersion();
        if (version) {
            handleVersionChange(version);
            return;
        }
    }
}

function startIntervalPolling() {
    setInterval(async () => {
        const version = await fetchVersion();
        if (version) handleVersionChange(version);
    }, POLL_INTERVAL_MS);
}

function showUpdateNotification() {
    document.body.classList.add('update-available');
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
