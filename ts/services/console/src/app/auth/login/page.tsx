'use client';

import { createSignupPasskey } from '@/lib/turnkey';
import { Button, Container, Divider, Paper, Stack, Text, TextInput, Title } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useRouter } from 'next/navigation';
import { type ReactElement, useState } from 'react';
import { mutate } from 'swr';

export default function LoginPage(): ReactElement {
    const router = useRouter();
    const [loading, setLoading] = useState(false);
    const [mode, setMode] = useState<'login' | 'signup'>('login');
    const [loginError, setLoginError] = useState('');

    const signupForm = useForm({
        initialValues: { email: '', displayName: '' },
        validate: {
            email: (value: string) =>
                /^\S+@\S+\.\S+$/.test(value) ? null : 'Invalid email address',
            displayName: (value: string) =>
                value.trim().length > 0 ? null : 'Display name is required',
        },
    });

    async function handleLogin(): Promise<void> {
        setLoading(true);
        setLoginError('');
        try {
            // Step 1: Get challenge from backend
            const challengeResp = await fetch('/api/auth/challenges', {
                method: 'POST',
                credentials: 'include',
            });
            if (!challengeResp.ok) {
                throw new Error('Failed to get challenge');
            }
            const { challenge } = (await challengeResp.json()) as { challenge: string };

            // Step 2: Sign challenge with passkey via navigator.credentials.get
            // This triggers the browser passkey UI
            const credential = await navigator.credentials.get({
                publicKey: {
                    challenge: new TextEncoder().encode(challenge),
                    rpId: window.location.hostname,
                    userVerification: 'required',
                },
            });

            if (!credential) {
                throw new Error('No credential returned');
            }

            // Step 3: Send the signed challenge to our session API
            const sessionResp = await fetch('/api/auth/sessions', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                    challenge,
                    credential: {
                        id: credential.id,
                        type: credential.type,
                        rawId: arrayBufferToBase64((credential as PublicKeyCredential).rawId),
                        response: {
                            authenticatorData: arrayBufferToBase64(
                                (
                                    (credential as PublicKeyCredential)
                                        .response as AuthenticatorAssertionResponse
                                ).authenticatorData,
                            ),
                            clientDataJSON: arrayBufferToBase64(
                                (credential as PublicKeyCredential).response.clientDataJSON,
                            ),
                            signature: arrayBufferToBase64(
                                (
                                    (credential as PublicKeyCredential)
                                        .response as AuthenticatorAssertionResponse
                                ).signature,
                            ),
                        },
                    },
                }),
            });

            if (!sessionResp.ok) {
                const body: unknown = await sessionResp.json().catch(() => null);
                const msg =
                    body !== null &&
                    typeof body === 'object' &&
                    'error' in body &&
                    typeof (body as Record<string, unknown>).error === 'string'
                        ? ((body as Record<string, unknown>).error as string)
                        : 'Login failed';
                throw new Error(msg);
            }

            await mutate('/api/auth/me');
            await mutate('/api/proxy/auth/orgs');
            router.push('/dashboard');
        } catch (err) {
            setLoginError(err instanceof Error ? err.message : 'Authentication failed');
        } finally {
            setLoading(false);
        }
    }

    async function handleSignup(values: {
        email: string;
        displayName: string;
    }): Promise<void> {
        setLoading(true);
        signupForm.clearErrors();
        try {
            // Step 1: Create passkey attestation client-side
            const attestation = await createSignupPasskey({
                email: values.email,
                displayName: values.displayName,
            });

            // Step 2: Send to our backend API route
            const resp = await fetch('/api/auth/people', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                    displayName: values.displayName,
                    email: values.email,
                    attestation,
                }),
            });

            if (!resp.ok) {
                const body: unknown = await resp.json().catch(() => null);
                const msg =
                    body !== null &&
                    typeof body === 'object' &&
                    'error' in body &&
                    typeof (body as Record<string, unknown>).error === 'string'
                        ? ((body as Record<string, unknown>).error as string)
                        : 'Signup failed';
                throw new Error(msg);
            }

            await mutate('/api/auth/me');
            await mutate('/api/proxy/auth/orgs');
            router.push('/dashboard');
        } catch (err) {
            signupForm.setFieldError('email', err instanceof Error ? err.message : 'Signup failed');
        } finally {
            setLoading(false);
        }
    }

    if (mode === 'signup') {
        return (
            <Container size="xs" py="xl">
                <Paper shadow="md" p="xl" radius="md" mt={100}>
                    <form onSubmit={signupForm.onSubmit(handleSignup)}>
                        <Stack gap="lg" align="center">
                            <Title order={2}>Create Account</Title>
                            <Text c="dimmed" ta="center">
                                Create a new EVMAuth account with a passkey.
                            </Text>

                            <TextInput
                                label="Display Name"
                                placeholder="Alice Adams"
                                required
                                w="100%"
                                size="md"
                                {...signupForm.getInputProps('displayName')}
                            />

                            <TextInput
                                type="email"
                                label="Email"
                                placeholder="you@example.com"
                                required
                                w="100%"
                                size="md"
                                {...signupForm.getInputProps('email')}
                            />

                            <Button type="submit" fullWidth size="lg" loading={loading}>
                                Create Account with Passkey
                            </Button>

                            <Divider w="100%" />

                            <Button
                                variant="subtle"
                                onClick={() => setMode('login')}
                                disabled={loading}
                            >
                                Already have an account? Sign in
                            </Button>
                        </Stack>
                    </form>
                </Paper>
            </Container>
        );
    }

    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <Stack gap="lg" align="center">
                    <Title order={2}>Sign in to EVMAuth</Title>
                    <Text c="dimmed" ta="center">
                        Sign in with your passkey to access the dashboard.
                    </Text>

                    {loginError && (
                        <Text c="red" size="sm">
                            {loginError}
                        </Text>
                    )}

                    <Button fullWidth size="lg" loading={loading} onClick={handleLogin}>
                        Sign in with Passkey
                    </Button>

                    <Divider w="100%" />

                    <Button variant="subtle" onClick={() => setMode('signup')} disabled={loading}>
                        New here? Create an account
                    </Button>
                </Stack>
            </Paper>
        </Container>
    );
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (const byte of bytes) {
        binary += String.fromCharCode(byte);
    }
    return btoa(binary);
}
