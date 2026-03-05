type NetworkConnection = {
    saveData?: boolean;
    effectiveType?: string;
};

type NavigatorWithConnection = Navigator & {
    connection?: NetworkConnection;
};

const DISALLOWED_CONNECTION_TYPES = new Set(['slow-2g', '2g', '3g']);

export function shouldPrefetchOnCurrentConnection(): boolean {
    if (typeof navigator === 'undefined') {
        return true;
    }

    const connection = (navigator as NavigatorWithConnection).connection;
    if (!connection) {
        return true;
    }

    if (connection.saveData) {
        return false;
    }

    const effectiveType = connection.effectiveType;
    if (!effectiveType) {
        return true;
    }

    return !DISALLOWED_CONNECTION_TYPES.has(effectiveType);
}
