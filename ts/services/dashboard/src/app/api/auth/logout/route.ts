import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL ?? 'http://gateway:8000';

export async function POST(request: Request) {
    // Forward the session cookie to the backend logout endpoint
    const cookieHeader = request.headers.get('Cookie');
    const headers = new Headers({ 'Content-Type': 'application/json' });
    if (cookieHeader) {
        headers.set('Cookie', cookieHeader);
    }

    const logoutResponse = await fetch(`${BACKEND_URL}/auth/auth/logout`, {
        method: 'POST',
        headers,
    });

    // Destroy iron-session regardless of backend response
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);
    session.destroy();

    // Forward Set-Cookie from backend (clears the session cookie)
    const responseHeaders = new Headers();
    const setCookies = logoutResponse.headers.getSetCookie();
    for (const setCookie of setCookies) {
        responseHeaders.append('Set-Cookie', setCookie);
    }

    return NextResponse.json({ success: true }, { headers: responseHeaders });
}
