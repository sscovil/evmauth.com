'use client';

import type { SessionData } from '@/lib/session';
import type {
    AppRegistrationResponse,
    ContractResponse,
    OrgResponse,
    PaginatedResponse,
    RoleGrantResponse,
} from '@/types/api';
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
    return useSWR<PaginatedResponse<OrgResponse>>('/api/proxy/auth/orgs', fetcher);
}

export function useOrg(orgId: string): SWRResponse<OrgResponse, Error> {
    return useSWR<OrgResponse>(`/api/proxy/auth/orgs/${orgId}`, fetcher);
}

export function useApps(
    orgId: string,
): SWRResponse<PaginatedResponse<AppRegistrationResponse>, Error> {
    return useSWR<PaginatedResponse<AppRegistrationResponse>>(
        `/api/proxy/registry/orgs/${orgId}/apps`,
        fetcher,
    );
}

export function useApp(orgId: string, appId: string): SWRResponse<AppRegistrationResponse, Error> {
    return useSWR<AppRegistrationResponse>(
        `/api/proxy/registry/orgs/${orgId}/apps/${appId}`,
        fetcher,
    );
}

export function useContracts(
    orgId: string,
): SWRResponse<PaginatedResponse<ContractResponse>, Error> {
    return useSWR<PaginatedResponse<ContractResponse>>(
        `/api/proxy/registry/orgs/${orgId}/contracts`,
        fetcher,
    );
}

export function useContract(
    orgId: string,
    contractId: string,
): SWRResponse<ContractResponse, Error> {
    return useSWR<ContractResponse>(
        `/api/proxy/registry/orgs/${orgId}/contracts/${contractId}`,
        fetcher,
    );
}

export function useRoleGrants(
    orgId: string,
    contractId: string,
): SWRResponse<RoleGrantResponse[], Error> {
    return useSWR<RoleGrantResponse[]>(
        `/api/proxy/registry/orgs/${orgId}/contracts/${contractId}/roles`,
        fetcher,
    );
}
