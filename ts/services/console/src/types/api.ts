export interface PersonResponse {
    id: string;
    display_name: string;
    description: string | null;
    auth_provider_name: string;
    auth_provider_ref: string;
    primary_email: string;
    created_at: string;
    updated_at: string;
}

export interface OrgResponse {
    id: string;
    display_name: string;
    description: string | null;
    owner_id: string;
    visibility: 'personal' | 'private' | 'public';
    created_at: string;
    updated_at: string;
}

export interface TokenResponse {
    access_token: string;
    token_type: string;
    expires_in: number;
}

export interface AppRegistrationResponse {
    id: string;
    org_id: string;
    name: string;
    client_id: string;
    callback_urls: string[];
    relevant_token_ids: number[];
    created_at: string;
    updated_at: string;
}

export interface ContractResponse {
    id: string;
    org_id: string;
    app_registration_id: string | null;
    name: string;
    address: string;
    chain_id: string;
    beacon_address: string;
    deploy_tx_hash: string;
    deployed_at: string;
    created_at: string;
    updated_at: string;
}

export interface RoleGrantResponse {
    id: string;
    contract_id: string;
    role: string;
    grant_tx_hash: string;
    revoke_tx_hash: string | null;
    active: boolean;
    granted_at: string;
    revoked_at: string | null;
    created_at: string;
    updated_at: string;
}

export interface PageInfo {
    hasNextPage: boolean;
    hasPreviousPage: boolean;
    startCursor: string | null;
    endCursor: string | null;
}

export interface Edge<T> {
    node: T;
    cursor: string;
}

export interface PaginatedResponse<T> {
    edges: Edge<T>[];
    pageInfo: PageInfo;
}
