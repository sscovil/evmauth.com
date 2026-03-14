'use client';

import type { SessionData } from '@/lib/session';
import type { OrgResponse, PaginatedResponse } from '@/types/api';
import type { SWRResponse } from 'swr';
import useSWR from 'swr';

async function fetcher<T>(url: string): Promise<T> {
    const response = await fetch(url, { credentials: 'include' });

    if (!response.ok) {
        const error = await response.json().catch(() => ({ error: 'Request failed' }));
        throw new Error((error as { error: string }).error ?? `HTTP ${response.status}`);
    }

    return response.json() as Promise<T>;
}

export function useMe(): SWRResponse<SessionData, Error> {
    return useSWR<SessionData>('/api/auth/me', fetcher);
}

export function useOrgs(): SWRResponse<PaginatedResponse<OrgResponse>, Error> {
    return useSWR<PaginatedResponse<OrgResponse>>('/api/proxy/auth/orgs', fetcher);
}
