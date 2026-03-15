import { config } from '@/lib/config';
import { PersonResponseSchema, TokenResponseSchema } from '@/lib/schemas';
import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';
import { z } from 'zod';

const SignupRequestSchema = z.object({
    displayName: z.string().min(1),
    email: z.string().email(),
});

export async function POST(request: Request): Promise<NextResponse> {
    const parsed = SignupRequestSchema.safeParse(await request.json());
    if (!parsed.success) {
        return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
    }
    const { displayName, email } = parsed.data;

    // Call backend signup
    const signupResponse = await fetch(`${config.backendUrl}/auth/auth/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            display_name: displayName,
            primary_email: email,
            auth_provider_name: 'email',
            auth_provider_ref: email,
        }),
    });

    if (!signupResponse.ok) {
        const error = await signupResponse.json().catch(() => ({ error: 'Signup failed' }));
        return NextResponse.json(error, { status: signupResponse.status });
    }

    let tokenData: z.infer<typeof TokenResponseSchema>;
    try {
        tokenData = TokenResponseSchema.parse(await signupResponse.json());
    } catch {
        return NextResponse.json({ error: 'Invalid response from auth service' }, { status: 502 });
    }

    // Fetch the full person record using the new session
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
