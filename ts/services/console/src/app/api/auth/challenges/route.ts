import { config } from '@/lib/config';
import { NextResponse } from 'next/server';

export async function POST(): Promise<NextResponse> {
    const response = await fetch(`${config.backendUrl}/auth/challenges`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
    });

    if (!response.ok) {
        return NextResponse.json(
            { error: 'Failed to create challenge' },
            { status: response.status },
        );
    }

    const data: unknown = await response.json();
    return NextResponse.json(data);
}
