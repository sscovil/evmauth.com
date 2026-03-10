import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import type { PersonResponse, TokenResponse } from '@/types/api';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL ?? 'http://gateway:8000';

export async function POST(request: Request) {
    const body = (await request.json()) as { email: string };

    // Call backend login
    const loginResponse = await fetch(`${BACKEND_URL}/auth/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            primary_email: body.email,
            auth_provider_name: 'email',
            auth_provider_ref: body.email,
        }),
    });

    if (!loginResponse.ok) {
        const error = await loginResponse.json().catch(() => ({ error: 'Login failed' }));
        return NextResponse.json(error, { status: loginResponse.status });
    }

    const tokenData = (await loginResponse.json()) as TokenResponse;

    // Fetch the full person record using the session
    const meResponse = await fetch(`${BACKEND_URL}/auth/me`, {
        headers: {
            Authorization: `${tokenData.token_type} ${tokenData.access_token}`,
        },
    });

    if (!meResponse.ok) {
        return NextResponse.json({ error: 'Failed to fetch user profile' }, { status: 500 });
    }

    const person = (await meResponse.json()) as PersonResponse;

    // Create iron-session
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);
    session.personId = person.id;
    session.email = person.primary_email;
    session.displayName = person.display_name;
    await session.save();

    // Build response and forward Set-Cookie from backend
    const responseHeaders = new Headers();
    const setCookies = loginResponse.headers.getSetCookie();
    for (const setCookie of setCookies) {
        responseHeaders.append('Set-Cookie', setCookie);
    }

    return NextResponse.json({ success: true }, { headers: responseHeaders });
}
