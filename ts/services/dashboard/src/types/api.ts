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
