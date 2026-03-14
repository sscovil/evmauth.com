import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';

export async function GET(): Promise<NextResponse> {
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);

    if (!session.personId) {
        return NextResponse.json({ error: 'Not authenticated' }, { status: 401 });
    }

    return NextResponse.json({
        personId: session.personId,
        email: session.email,
        displayName: session.displayName,
    });
}
