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

    await fetch(`${config.backendUrl}/auth/auth/logout`, {
        method: 'POST',
        headers,
    });

    // Destroy iron-session regardless of backend response
    session.destroy();

    return NextResponse.json({ success: true });
}
