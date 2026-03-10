import type { SessionOptions } from 'iron-session';

export interface SessionData {
    personId: string;
    email: string;
    displayName: string;
}

export const sessionOptions: SessionOptions = {
    password:
        process.env.SESSION_SECRET ??
        'this-is-a-development-secret-that-must-be-changed-in-production',
    cookieName: 'evmauth-dashboard',
    cookieOptions: {
        secure: process.env.NODE_ENV === 'production',
        httpOnly: true,
        sameSite: 'lax',
        maxAge: 28800, // 8 hours, matches backend JWT
    },
};
