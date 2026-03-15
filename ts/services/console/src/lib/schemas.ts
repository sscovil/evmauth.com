import { z } from 'zod';

export const TokenResponseSchema = z.object({
    access_token: z.string(),
    token_type: z.string(),
    expires_in: z.number(),
});

export const PersonResponseSchema = z.object({
    id: z.string(),
    display_name: z.string(),
    description: z.string().nullable(),
    auth_provider_name: z.string(),
    auth_provider_ref: z.string(),
    primary_email: z.string(),
    created_at: z.string(),
    updated_at: z.string(),
});
