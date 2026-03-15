import { config } from '@/lib/config';
import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { cookies } from 'next/headers';
import { type NextRequest, NextResponse } from 'next/server';

async function proxyRequest(
    request: NextRequest,
    params: Promise<{ path: string[] }>,
): Promise<NextResponse> {
    const session = await getIronSession<SessionData>(await cookies(), sessionOptions);
    if (!session.personId) {
        return NextResponse.json({ error: 'Not authenticated' }, { status: 401 });
    }

    const { path } = await params;
    const targetPath = path.join('/');
    const url = `${config.backendUrl}/${targetPath}${request.nextUrl.search}`;

    const headers = new Headers();
    headers.set('Content-Type', request.headers.get('Content-Type') ?? 'application/json');
    headers.set('Accept', request.headers.get('Accept') ?? 'application/json');
    headers.set('X-Person-Id', session.personId);

    // Forward authorization if present
    const authorization = request.headers.get('Authorization');
    if (authorization) {
        headers.set('Authorization', authorization);
    }

    const fetchOptions: RequestInit = {
        method: request.method,
        headers,
    };

    if (request.method !== 'GET' && request.method !== 'HEAD') {
        fetchOptions.body = await request.text();
    }

    const response = await fetch(url, fetchOptions);

    const responseHeaders = new Headers({
        'Content-Type': response.headers.get('Content-Type') ?? 'application/json',
    });

    return new NextResponse(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: responseHeaders,
    });
}

export async function GET(
    request: NextRequest,
    context: { params: Promise<{ path: string[] }> },
): Promise<NextResponse> {
    return proxyRequest(request, context.params);
}

export async function POST(
    request: NextRequest,
    context: { params: Promise<{ path: string[] }> },
): Promise<NextResponse> {
    return proxyRequest(request, context.params);
}

export async function PATCH(
    request: NextRequest,
    context: { params: Promise<{ path: string[] }> },
): Promise<NextResponse> {
    return proxyRequest(request, context.params);
}

export async function PUT(
    request: NextRequest,
    context: { params: Promise<{ path: string[] }> },
): Promise<NextResponse> {
    return proxyRequest(request, context.params);
}

export async function DELETE(
    request: NextRequest,
    context: { params: Promise<{ path: string[] }> },
): Promise<NextResponse> {
    return proxyRequest(request, context.params);
}
