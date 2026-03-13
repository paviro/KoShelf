export class ApiHttpError extends Error {
    readonly status: number;
    readonly url: string;

    constructor(url: string, status: number) {
        super(`Failed to fetch ${url} (${status})`);
        this.name = 'ApiHttpError';
        this.status = status;
        this.url = url;
    }
}

export function isApiHttpError(error: unknown): error is ApiHttpError {
    return error instanceof ApiHttpError;
}

export async function fetchJson(url: string): Promise<unknown> {
    const response = await fetch(url, {
        method: 'GET',
        headers: { Accept: 'application/json' },
    });

    if (!response.ok) {
        throw new ApiHttpError(url, response.status);
    }

    return response.json();
}
