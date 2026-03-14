import { config } from '@/lib/config';
import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

export async function POST(): Promise<NextResponse> {
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);

    // Call backend logout with person identification
    const headers = new Headers({ 'Content-Type': 'application/json' });
    if (session.personId) {
        headers.set('X-Person-Id', session.personId);
    }

    const logoutResponse = await fetch(`${config.backendUrl}/auth/auth/logout`, {
        method: 'POST',
        headers,
    });

    // Destroy iron-session regardless of backend response
    session.destroy();

    // Forward Set-Cookie from backend (clears the session cookie)
    const responseHeaders = new Headers();
    const setCookies = logoutResponse.headers.getSetCookie();
    for (const setCookie of setCookies) {
        responseHeaders.append('Set-Cookie', setCookie);
    }

    return NextResponse.json({ success: true }, { headers: responseHeaders });
}
