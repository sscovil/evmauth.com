const API_BASE = '/api/proxy';

interface FetchOptions {
    method?: string;
    body?: unknown;
    headers?: Record<string, string>;
}

function getErrorMessage(body: unknown, fallback: string): string {
    if (body !== null && typeof body === 'object' && 'error' in body) {
        const value = (body as Record<string, unknown>).error;
        if (typeof value === 'string') return value;
    }
    return fallback;
}

async function request<T>(path: string, options: FetchOptions = {}): Promise<T> {
    const url = `${API_BASE}${path}`;
    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...options.headers,
    };

    const response = await fetch(url, {
        method: options.method ?? 'GET',
        headers,
        body: options.body ? JSON.stringify(options.body) : undefined,
        credentials: 'include',
    });

    if (!response.ok) {
        const errorBody: unknown = await response.json().catch(() => null);
        throw new Error(getErrorMessage(errorBody, `HTTP ${response.status}`));
    }

    // Trusted type assertion: callers provide T matching the expected backend response shape
    return response.json() as Promise<T>;
}

interface ApiClient {
    get: <T>(path: string) => Promise<T>;
    post: <T>(path: string, body: unknown) => Promise<T>;
    patch: <T>(path: string, body: unknown) => Promise<T>;
    delete: <T>(path: string) => Promise<T>;
}

export const api: ApiClient = {
    get: <T>(path: string): Promise<T> => request<T>(path),
    post: <T>(path: string, body: unknown): Promise<T> =>
        request<T>(path, { method: 'POST', body }),
    patch: <T>(path: string, body: unknown): Promise<T> =>
        request<T>(path, { method: 'PATCH', body }),
    delete: <T>(path: string): Promise<T> => request<T>(path, { method: 'DELETE' }),
};
