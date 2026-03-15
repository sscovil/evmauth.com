'use client';

import type { SessionData } from '@/lib/session';
import type { OrgResponse, PaginatedResponse } from '@/types/api';
import type { SWRResponse } from 'swr';
import useSWR from 'swr';

async function fetcher<T>(url: string): Promise<T> {
    const response = await fetch(url, { credentials: 'include' });

    if (!response.ok) {
        const errorBody: unknown = await response.json().catch(() => null);
        const message =
            errorBody !== null &&
            typeof errorBody === 'object' &&
            'error' in errorBody &&
            typeof (errorBody as Record<string, unknown>).error === 'string'
                ? ((errorBody as Record<string, unknown>).error as string)
                : `HTTP ${response.status}`;
        throw new Error(message);
    }

    return response.json() as Promise<T>;
}

export function useMe(): SWRResponse<SessionData, Error> {
    // Trusted type assertion: /api/auth/me returns SessionData from iron-session server route
    return useSWR<SessionData>('/api/auth/me', fetcher);
}

export function useOrgs(): SWRResponse<PaginatedResponse<OrgResponse>, Error> {
    // Trusted type assertion: backend response matches PaginatedResponse<OrgResponse>
    return useSWR<PaginatedResponse<OrgResponse>>('/api/proxy/auth/orgs', fetcher);
}
