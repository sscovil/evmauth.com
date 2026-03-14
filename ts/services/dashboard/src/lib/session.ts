import { config } from '@/lib/config';
import type { SessionOptions } from 'iron-session';

export interface SessionData {
    personId: string;
    email: string;
    displayName: string;
}

export const sessionOptions: SessionOptions = {
    password: config.sessionSecret,
    cookieName: 'evmauth-dashboard',
    cookieOptions: {
        secure: config.isProduction,
        httpOnly: true,
        sameSite: 'strict',
        maxAge: 28800, // 8 hours, matches backend JWT
    },
};
