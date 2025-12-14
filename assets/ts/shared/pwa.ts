// PWA utilities - service worker registration and update notifications
import { StorageManager } from './storage-manager.js';

const KEYS = StorageManager.KEYS;
const RECOVERY_CONFIG = { maxRetries: 3, cooldownMs: 60000 };
const RECONNECT_DELAY_MS = 5000;
const POLL_INTERVAL_MS = 10000;
const SERVER_MODE: string = '{{SERVER_MODE}}'; // Injected at build time

// Version tracking state
let initialVersion: string | null = StorageManager.get<string>(KEYS.VERSION);
let lastNotifiedVersion: string | null = null; // Track which version we last triggered an update for

interface ServiceWorkerMessage {
    type: string;
    error?: string;
}

if ('serviceWorker' in navigator) {
    initializeServiceWorker();
}

async function initializeServiceWorker(): Promise<void> {
    try {
        await navigator.serviceWorker.register('/service-worker.js');
    } catch (error) {
        console.error('[PWA] Service Worker registration failed:', error);
        recoveryReload();
    }

    navigator.serviceWorker.addEventListener('message', handleServiceWorkerMessage);
    window.addEventListener('error', handleWindowError);
    window.addEventListener('unhandledrejection', handleUnhandledRejection);

    // Start version monitoring after page load
    setTimeout(startVersionMonitoring, 1000);
}

function handleServiceWorkerMessage({ data }: MessageEvent<ServiceWorkerMessage>): void {
    if (data?.type === 'CACHE_UPDATED') {
        showUpdateNotification();
    } else if (data?.type === 'CRITICAL_ERROR') {
        console.error('[PWA] Service Worker critical error:', data.error);
        recoveryReload();
    }
}

function handleWindowError(event: ErrorEvent): void {
    if (event.error || event.message) {
        console.error('[PWA] Window error detected:', event.error || event.message);
        recoveryReload();
    }
}

function handleUnhandledRejection(event: PromiseRejectionEvent): void {
    // Some runtime failures surface only as unhandled promise rejections.
    // Treat them as critical and trigger the same recovery path.
    if (event.reason) {
        console.error('[PWA] Unhandled rejection detected:', event.reason);
    } else {
        console.error('[PWA] Unhandled rejection detected');
    }
    recoveryReload();
}

function recoveryReload(): void {
    const now = Date.now();
    const lastReload = parseInt(StorageManager.get<string>(KEYS.LAST_RELOAD) || '0');
    let count = parseInt(StorageManager.get<string>(KEYS.RELOAD_COUNT) || '0');

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
async function startVersionMonitoring(): Promise<void> {
    const version = await fetchVersion();
    if (version) handleVersionChange(version);

    if (SERVER_MODE === 'internal') {
        startLongPolling();
    } else {
        startIntervalPolling();
    }
}

async function fetchVersion(): Promise<string | null> {
    try {
        const response = await fetch('/version.txt', { cache: 'no-store' });
        return response.ok ? (await response.text()).trim() : null;
    } catch {
        return null;
    }
}

function handleVersionChange(version: string): void {
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

function requestCacheUpdate(): void {
    if (navigator.serviceWorker.controller) {
        console.log('[PWA] Sending CHECK_FOR_UPDATES to service worker');
        navigator.serviceWorker.controller.postMessage({ type: 'CHECK_FOR_UPDATES' });
    } else {
        console.log('[PWA] No SW controller, showing notification directly');
        showUpdateNotification();
    }
}

async function startLongPolling(): Promise<void> {
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

async function waitForServerRecovery(): Promise<void> {
    while (true) {
        await sleep(RECONNECT_DELAY_MS);
        const version = await fetchVersion();
        if (version) {
            handleVersionChange(version);
            return;
        }
    }
}

function startIntervalPolling(): void {
    setInterval(async () => {
        const version = await fetchVersion();
        if (version) handleVersionChange(version);
    }, POLL_INTERVAL_MS);
}

function showUpdateNotification(): void {
    document.body.classList.add('update-available');
}

function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}
