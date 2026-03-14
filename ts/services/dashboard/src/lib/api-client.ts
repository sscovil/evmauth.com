const API_BASE = '/api/proxy';

interface FetchOptions {
    method?: string;
    body?: unknown;
    headers?: Record<string, string>;
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
        const error = await response.json().catch(() => ({ error: 'Request failed' }));
        throw new Error((error as { error: string }).error ?? `HTTP ${response.status}`);
    }

    return response.json() as Promise<T>;
}

/**
 * Authenticate a user by email. Attempts login first; if the user does not
 * exist (401/404), falls back to signup automatically.
 */
export async function authenticate(email: string): Promise<void> {
    const loginResponse = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email }),
        credentials: 'include',
    });

    if (loginResponse.ok) {
        return;
    }

    if (loginResponse.status === 401 || loginResponse.status === 404) {
        const displayName = email.split('@')[0] ?? email;
        const signupResponse = await fetch('/api/auth/signup', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ displayName, email }),
            credentials: 'include',
        });

        if (signupResponse.ok) {
            return;
        }

        const signupError = (await signupResponse.json().catch(() => ({
            error: 'Signup failed',
        }))) as { error: string };
        throw new Error(signupError.error);
    }

    const loginError = (await loginResponse.json().catch(() => ({
        error: 'Login failed',
    }))) as { error: string };
    throw new Error(loginError.error);
}

export const api = {
    get: <T>(path: string): Promise<T> => request<T>(path),
    post: <T>(path: string, body: unknown): Promise<T> =>
        request<T>(path, { method: 'POST', body }),
    patch: <T>(path: string, body: unknown): Promise<T> =>
        request<T>(path, { method: 'PATCH', body }),
    delete: <T>(path: string): Promise<T> => request<T>(path, { method: 'DELETE' }),
};
