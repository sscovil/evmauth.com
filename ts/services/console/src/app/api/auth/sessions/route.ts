import { config } from '@/lib/config';
import { PersonResponseSchema, TokenResponseSchema } from '@/lib/schemas';
import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';
import { z } from 'zod';

const LoginRequestSchema = z.object({
    challenge: z.string(),
    credential: z.object({
        id: z.string(),
        type: z.string(),
        rawId: z.string(),
        response: z.object({
            authenticatorData: z.string(),
            clientDataJSON: z.string(),
            signature: z.string(),
        }),
    }),
});

export async function POST(request: Request): Promise<NextResponse> {
    const parsed = LoginRequestSchema.safeParse(await request.json());
    if (!parsed.success) {
        return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
    }
    const { challenge, credential } = parsed.data;

    // The passkey assertion was completed client-side. We now need to verify it
    // by forwarding the challenge + signature to the backend, which will:
    // 1. Consume the challenge nonce from Redis
    // 2. Recover the signer from the assertion signature
    // 3. Look up the person by wallet address
    // 4. Verify on-chain token balance
    // 5. Issue a session JWT

    // For passkey-based login, the backend needs the wallet_address and a
    // signature of the challenge. The Turnkey wallet signature is performed
    // on the Turnkey infrastructure. Since we're using passkeys for identity
    // verification (not wallet signing), we forward the credential assertion
    // to the backend which verifies the passkey via Turnkey's API.
    //
    // However, the current backend login endpoint expects { wallet_address, signature, challenge }.
    // In a passkey flow, the "signature" comes from the WebAuthn assertion.
    // We'll forward the credential data and let the backend resolve it.
    const loginResponse = await fetch(`${config.backendUrl}/auth/sessions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            wallet_address: credential.id, // credential ID maps to user's Turnkey wallet
            signature: credential.response.signature,
            challenge,
        }),
    });

    if (!loginResponse.ok) {
        const error = await loginResponse.json().catch(() => ({ error: 'Login failed' }));
        return NextResponse.json(error, { status: loginResponse.status });
    }

    let tokenData: z.infer<typeof TokenResponseSchema>;
    try {
        tokenData = TokenResponseSchema.parse(await loginResponse.json());
    } catch {
        return NextResponse.json({ error: 'Invalid response from auth service' }, { status: 502 });
    }

    // Fetch the full person record
    const meResponse = await fetch(`${config.backendUrl}/auth/me`, {
        headers: {
            Authorization: `${tokenData.token_type} ${tokenData.access_token}`,
        },
    });

    if (!meResponse.ok) {
        return NextResponse.json({ error: 'Failed to fetch user profile' }, { status: 500 });
    }

    let person: z.infer<typeof PersonResponseSchema>;
    try {
        person = PersonResponseSchema.parse(await meResponse.json());
    } catch {
        return NextResponse.json({ error: 'Invalid response from auth service' }, { status: 502 });
    }

    // Create iron-session
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);
    session.personId = person.id;
    session.email = person.primary_email;
    session.displayName = person.display_name;
    await session.save();

    return NextResponse.json({ success: true });
}

export async function DELETE(): Promise<NextResponse> {
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);

    // Call backend logout with person identification
    const headers = new Headers({ 'Content-Type': 'application/json' });
    if (session.personId) {
        headers.set('X-Person-Id', session.personId);
    }

    await fetch(`${config.backendUrl}/auth/sessions`, {
        method: 'DELETE',
        headers,
    });

    // Destroy iron-session regardless of backend response
    session.destroy();

    return NextResponse.json({ success: true });
}
