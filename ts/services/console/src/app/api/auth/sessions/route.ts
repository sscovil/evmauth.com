import { config } from '@/lib/config';
import { PersonResponseSchema, TokenResponseSchema } from '@/lib/schemas';
import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';
import { z } from 'zod';

const LoginRequestSchema = z.object({ email: z.string().email() });

export async function POST(request: Request): Promise<NextResponse> {
    const parsed = LoginRequestSchema.safeParse(await request.json());
    if (!parsed.success) {
        return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
    }
    const { email } = parsed.data;

    // Call backend login
    const loginResponse = await fetch(`${config.backendUrl}/auth/sessions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            primary_email: email,
            auth_provider_name: 'email',
            auth_provider_ref: email,
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

    // Fetch the full person record using the session
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
