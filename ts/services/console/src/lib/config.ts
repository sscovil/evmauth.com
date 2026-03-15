interface Config {
    readonly sessionSecret: string;
    readonly backendUrl: string;
    readonly isProduction: boolean;
}

export const config: Config = {
    get sessionSecret(): string {
        const value = process.env.SESSION_SECRET;
        if (!value) {
            throw new Error('SESSION_SECRET environment variable is required');
        }
        return value;
    },
    get backendUrl(): string {
        return process.env.BACKEND_URL ?? 'http://gateway:8000';
    },
    get isProduction(): boolean {
        return process.env.NODE_ENV === 'production';
    },
};
