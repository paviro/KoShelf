type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

export type FetchJsonOptions = {
    method?: HttpMethod;
    headers?: Record<string, string>;
    body?: string;
    redirectOnUnauthorized?: boolean;
    cache?: RequestCache;
};

type ApiHttpErrorOptions = {
    code?: string;
    apiMessage?: string;
    details?: Record<string, unknown>;
    retryAfterSeconds?: number;
};

type ApiErrorEnvelope = {
    error?: {
        code?: string;
        message?: string;
        details?: unknown;
    };
};

export class ApiHttpError extends Error {
    readonly status: number;
    readonly url: string;
    readonly code?: string;
    readonly apiMessage?: string;
    readonly details?: Record<string, unknown>;
    readonly retryAfterSeconds?: number;

    constructor(
        url: string,
        status: number,
        options: ApiHttpErrorOptions = {},
    ) {
        const details = options.apiMessage ? `: ${options.apiMessage}` : '';
        super(`Failed to fetch ${url} (${status})${details}`);
        this.name = 'ApiHttpError';
        this.status = status;
        this.url = url;
        this.code = options.code;
        this.apiMessage = options.apiMessage;
        this.details = options.details;
        this.retryAfterSeconds = options.retryAfterSeconds;
    }
}

export function isApiHttpError(error: unknown): error is ApiHttpError {
    return error instanceof ApiHttpError;
}

export function isLoginHashRoute(): boolean {
    if (typeof window === 'undefined') {
        return false;
    }

    return window.location.hash.startsWith('#/login');
}

export function redirectToLogin(): void {
    if (typeof window === 'undefined') {
        return;
    }

    if (!isLoginHashRoute()) {
        window.location.replace('/#/login');
    }
}

function parseRetryAfterSeconds(
    headerValue: string | null,
): number | undefined {
    if (!headerValue) {
        return undefined;
    }

    const parsed = Number.parseInt(headerValue, 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

async function parseErrorEnvelope(response: Response): Promise<{
    code?: string;
    apiMessage?: string;
    details?: Record<string, unknown>;
}> {
    const contentType = response.headers.get('content-type') ?? '';
    if (!contentType.toLowerCase().includes('application/json')) {
        return {};
    }

    try {
        const payload = (await response.json()) as ApiErrorEnvelope;
        const code = payload.error?.code;
        const apiMessage = payload.error?.message;
        const rawDetails = payload.error?.details;
        const details =
            rawDetails &&
            typeof rawDetails === 'object' &&
            !Array.isArray(rawDetails)
                ? (rawDetails as Record<string, unknown>)
                : undefined;

        return {
            code: typeof code === 'string' ? code : undefined,
            apiMessage: typeof apiMessage === 'string' ? apiMessage : undefined,
            details,
        };
    } catch {
        return {};
    }
}

export async function fetchJson(
    url: string,
    options: FetchJsonOptions = {},
): Promise<unknown> {
    const response = await fetch(url, {
        method: options.method ?? 'GET',
        headers: {
            Accept: 'application/json',
            ...(options.headers ?? {}),
        },
        body: options.body,
        cache: options.cache,
    });

    if (response.status === 401 && options.redirectOnUnauthorized !== false) {
        redirectToLogin();
        return new Promise<never>(() => {});
    }

    if (!response.ok) {
        const errorPayload = await parseErrorEnvelope(response);
        throw new ApiHttpError(url, response.status, {
            code: errorPayload.code,
            apiMessage: errorPayload.apiMessage,
            details: errorPayload.details,
            retryAfterSeconds: parseRetryAfterSeconds(
                response.headers.get('retry-after'),
            ),
        });
    }

    if (response.status === 204) {
        return null;
    }

    return response.json();
}
