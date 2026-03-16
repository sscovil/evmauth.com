'use client';

import {
    Alert,
    Button,
    Container,
    Loader,
    Paper,
    Stack,
    Text,
    TextInput,
    Title,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { useSearchParams } from 'next/navigation';
import { type ReactElement, Suspense, useEffect, useState } from 'react';

interface AppInfo {
    app_name: string;
    client_id: string;
    redirect_uri: string;
}

interface AuthenticateResponse {
    redirect_url: string;
}

function EndUserLoginForm(): ReactElement {
    const searchParams = useSearchParams();
    const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
    const [validationError, setValidationError] = useState('');
    const [loading, setLoading] = useState(false);
    const [validating, setValidating] = useState(true);

    const clientId = searchParams.get('client_id') ?? '';
    const redirectUri = searchParams.get('redirect_uri') ?? '';
    const state = searchParams.get('state') ?? '';
    const codeChallenge = searchParams.get('code_challenge') ?? '';
    const codeChallengeMethod = searchParams.get('code_challenge_method') ?? '';

    const form = useForm({
        initialValues: { email: '', displayName: '' },
        validate: {
            email: (value: string) =>
                /^\S+@\S+\.\S+$/.test(value) ? null : 'Invalid email address',
        },
    });

    useEffect(() => {
        if (!clientId || !redirectUri || !codeChallenge) {
            setValidationError(
                'Missing required parameters: client_id, redirect_uri, code_challenge',
            );
            setValidating(false);
            return;
        }

        if (codeChallengeMethod && codeChallengeMethod !== 'S256') {
            setValidationError('code_challenge_method must be S256');
            setValidating(false);
            return;
        }

        const params = new URLSearchParams({
            client_id: clientId,
            redirect_uri: redirectUri,
            code_challenge: codeChallenge,
            code_challenge_method: 'S256',
        });
        if (state) params.set('state', state);

        fetch(`/api/proxy/auth/auth/end-user/authorize?${params.toString()}`, {
            credentials: 'include',
        })
            .then(async (resp) => {
                if (!resp.ok) {
                    const body: unknown = await resp.json().catch(() => null);
                    const msg =
                        body !== null &&
                        typeof body === 'object' &&
                        'error' in body &&
                        typeof (body as Record<string, unknown>).error === 'string'
                            ? ((body as Record<string, unknown>).error as string)
                            : `Validation failed (${resp.status})`;
                    setValidationError(msg);
                    return;
                }
                const data = (await resp.json()) as AppInfo;
                setAppInfo(data);
            })
            .catch((e: Error) => {
                setValidationError(e.message);
            })
            .finally(() => {
                setValidating(false);
            });
    }, [clientId, redirectUri, codeChallenge, codeChallengeMethod, state]);

    async function handleSubmit(values: {
        email: string;
        displayName: string;
    }): Promise<void> {
        setLoading(true);
        form.clearErrors();

        try {
            const resp = await fetch('/api/proxy/auth/auth/end-user/authorize', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                    client_id: clientId,
                    redirect_uri: redirectUri,
                    code_challenge: codeChallenge,
                    state: state || undefined,
                    email: values.email,
                    display_name: values.displayName || undefined,
                    auth_provider_name: 'turnkey',
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
                        : 'Authentication failed';
                form.setFieldError('email', msg);
                return;
            }

            const data = (await resp.json()) as AuthenticateResponse;
            window.location.href = data.redirect_url;
        } catch (err) {
            form.setFieldError(
                'email',
                err instanceof Error ? err.message : 'Authentication failed',
            );
        } finally {
            setLoading(false);
        }
    }

    if (validating) {
        return (
            <Container size="xs" py="xl">
                <Stack align="center" mt={100}>
                    <Loader />
                    <Text c="dimmed">Validating authorization request...</Text>
                </Stack>
            </Container>
        );
    }

    if (validationError) {
        return (
            <Container size="xs" py="xl">
                <Alert color="red" title="Invalid Authorization Request" mt={100}>
                    {validationError}
                </Alert>
            </Container>
        );
    }

    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <form onSubmit={form.onSubmit(handleSubmit)}>
                    <Stack gap="lg" align="center">
                        <Title order={2}>Sign in</Title>
                        <Text c="dimmed" ta="center">
                            <strong>{appInfo?.app_name}</strong> is requesting access to your
                            EVMAuth account.
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
                            placeholder="Your name (optional, for new accounts)"
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
                            Continue
                        </Button>
                    </Stack>
                </form>
            </Paper>
        </Container>
    );
}

export default function EndUserLoginPage(): ReactElement {
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
            <EndUserLoginForm />
        </Suspense>
    );
}
