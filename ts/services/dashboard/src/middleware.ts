import type { SessionData } from '@/lib/session';
import { sessionOptions } from '@/lib/session';
import { getIronSession } from 'iron-session';
import { type NextRequest, NextResponse } from 'next/server';

export async function middleware(request: NextRequest) {
    const response = NextResponse.next();

    const session = await getIronSession<SessionData>(request, response, sessionOptions);
    const isAuthenticated = !!session.personId;
    const { pathname } = request.nextUrl;

    // Redirect authenticated users away from login page
    if (pathname === '/auth/login' && isAuthenticated) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
    }

    // Protect dashboard routes
    if (pathname.startsWith('/dashboard') && !isAuthenticated) {
        return NextResponse.redirect(new URL('/auth/login', request.url));
    }

    return response;
}

export const config = {
    matcher: ['/dashboard/:path*', '/auth/login'],
};
