'use client';

import {
    Alert,
    Button,
    Code,
    Container,
    Loader,
    Paper,
    Stack,
    Text,
    TextInput,
    Title,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { getWebAuthnAttestation } from '@turnkey/sdk-browser';
import { useSearchParams } from 'next/navigation';
import { type ReactElement, Suspense, useEffect, useState } from 'react';

interface CreateEndUserResponse {
    wallet_address: string;
    person_id: string;
}

function EndUserSignupForm(): ReactElement {
    const searchParams = useSearchParams();
    const [loading, setLoading] = useState(false);
    const [validating, setValidating] = useState(true);
    const [validationError, setValidationError] = useState('');
    const [result, setResult] = useState<CreateEndUserResponse | null>(null);

    const clientId = searchParams.get('client_id') ?? '';
    const callbackUrl = searchParams.get('callback_url') ?? '';

    const form = useForm({
        initialValues: { email: '', displayName: '' },
        validate: {
            email: (value: string) =>
                /^\S+@\S+\.\S+$/.test(value) ? null : 'Invalid email address',
        },
    });

    useEffect(() => {
        if (!clientId) {
            setValidationError('Missing required parameter: client_id');
        }
        setValidating(false);
    }, [clientId]);

    async function handleSubmit(values: {
        email: string;
        displayName: string;
    }): Promise<void> {
        setLoading(true);
        form.clearErrors();

        try {
            // Create passkey attestation
            const challenge = crypto.randomUUID();
            const attestation = await getWebAuthnAttestation({
                publicKey: {
                    rp: { id: window.location.hostname, name: 'EVMAuth' },
                    user: {
                        id: new TextEncoder().encode(challenge),
                        name: values.email,
                        displayName: values.displayName || values.email,
                    },
                    challenge: new TextEncoder().encode(challenge),
                    pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
                    authenticatorSelection: {
                        residentKey: 'preferred',
                        userVerification: 'required',
                    },
                },
            });

            // Submit to backend
            const resp = await fetch('/api/proxy/auth/end-users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                    email: values.email,
                    display_name: values.displayName || undefined,
                    client_id: clientId,
                    attestation: {
                        authenticator_name: `${values.displayName || values.email}-passkey`,
                        challenge,
                        attestation,
                    },
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
                form.setFieldError('email', msg);
                return;
            }

            const data = (await resp.json()) as CreateEndUserResponse;
            setResult(data);

            // If a callback URL is provided, redirect with the wallet address
            if (callbackUrl) {
                const separator = callbackUrl.includes('?') ? '&' : '?';
                window.location.href = `${callbackUrl}${separator}wallet_address=${data.wallet_address}`;
            }
        } catch (err) {
            form.setFieldError('email', err instanceof Error ? err.message : 'Signup failed');
        } finally {
            setLoading(false);
        }
    }

    if (validating) {
        return (
            <Container size="xs" py="xl">
                <Stack align="center" mt={100}>
                    <Loader />
                    <Text c="dimmed">Validating request...</Text>
                </Stack>
            </Container>
        );
    }

    if (validationError) {
        return (
            <Container size="xs" py="xl">
                <Alert color="red" title="Invalid Request" mt={100}>
                    {validationError}
                </Alert>
            </Container>
        );
    }

    if (result) {
        return (
            <Container size="xs" py="xl">
                <Paper shadow="md" p="xl" radius="md" mt={100}>
                    <Stack gap="lg" align="center">
                        <Title order={2}>Account Created</Title>
                        <Text c="dimmed" ta="center">
                            Your wallet address has been created.
                        </Text>
                        <div>
                            <Text size="sm" fw={500} mb={4}>
                                Wallet Address
                            </Text>
                            <Code>{result.wallet_address}</Code>
                        </div>
                    </Stack>
                </Paper>
            </Container>
        );
    }

    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <form onSubmit={form.onSubmit(handleSubmit)}>
                    <Stack gap="lg" align="center">
                        <Title order={2}>Create Account</Title>
                        <Text c="dimmed" ta="center">
                            Create an EVMAuth account with a passkey to get started.
                        </Text>

                        <TextInput
                            type="email"
                            label="Email"
                            placeholder="you@example.com"
                            required
                            w="100%"
                            size="md"
                            {...form.getInputProps('email')}
                        />

                        <TextInput
                            label="Display Name"
                            placeholder="Your name (optional)"
                            w="100%"
                            size="md"
                            {...form.getInputProps('displayName')}
                        />

                        <Button
                            type="submit"
                            fullWidth
                            size="lg"
                            loading={loading}
                            disabled={!form.values.email}
                        >
                            Create Account with Passkey
                        </Button>
                    </Stack>
                </form>
            </Paper>
        </Container>
    );
}

export default function EndUserSignupPage(): ReactElement {
    return (
        <Suspense
            fallback={
                <Container size="xs" py="xl">
                    <Stack align="center" mt={100}>
                        <Loader />
                    </Stack>
                </Container>
            }
        >
            <EndUserSignupForm />
        </Suspense>
    );
}
