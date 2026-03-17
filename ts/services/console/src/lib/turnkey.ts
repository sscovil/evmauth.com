import { TurnkeyPasskeyClient, getWebAuthnAttestation } from '@turnkey/sdk-browser';

const TURNKEY_API_BASE_URL = 'https://api.turnkey.com';

/**
 * Create a TurnkeyPasskeyClient for passkey-based authentication.
 * Used during signup to create a new passkey + sub-org.
 */
export function createPasskeyClient(): TurnkeyPasskeyClient {
    return new TurnkeyPasskeyClient({
        apiBaseUrl: TURNKEY_API_BASE_URL,
    });
}

/**
 * Create a passkey via WebAuthn for user signup.
 * Returns attestation data to send to the backend.
 */
export async function createSignupPasskey(params: {
    email: string;
    displayName: string;
}): Promise<{
    authenticator_name: string;
    challenge: string;
    attestation: unknown;
}> {
    const challenge = crypto.randomUUID();

    const attestation = await getWebAuthnAttestation({
        publicKey: {
            rp: { id: window.location.hostname, name: 'EVMAuth' },
            user: {
                id: new TextEncoder().encode(challenge),
                name: params.email,
                displayName: params.displayName,
            },
            challenge: new TextEncoder().encode(challenge),
            pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
            authenticatorSelection: {
                residentKey: 'preferred',
                userVerification: 'required',
            },
        },
    });

    return {
        authenticator_name: `${params.displayName}-passkey`,
        challenge,
        attestation,
    };
}

export { getWebAuthnAttestation };
