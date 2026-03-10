'use client';

import { Alert, Button, Container, Paper, Stack, Text, TextInput, Title } from '@mantine/core';
import { useRouter } from 'next/navigation';
import { type FormEvent, useState } from 'react';

export default function LoginPage() {
    const router = useRouter();
    const [email, setEmail] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    async function handleSubmit(e: FormEvent) {
        e.preventDefault();
        setError('');
        setLoading(true);

        try {
            // Try login first
            const loginResponse = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email }),
                credentials: 'include',
            });

            if (loginResponse.ok) {
                router.push('/dashboard');
                return;
            }

            // If login fails with 401/404, try signup
            if (loginResponse.status === 401 || loginResponse.status === 404) {
                const signupResponse = await fetch('/api/auth/signup', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ displayName: email.split('@')[0], email }),
                    credentials: 'include',
                });

                if (signupResponse.ok) {
                    router.push('/dashboard');
                    return;
                }

                const signupError = (await signupResponse.json().catch(() => ({
                    error: 'Signup failed',
                }))) as { error: string };
                setError(signupError.error);
                return;
            }

            const loginError = (await loginResponse.json().catch(() => ({
                error: 'Login failed',
            }))) as { error: string };
            setError(loginError.error);
        } catch {
            setError('An unexpected error occurred. Please try again.');
        } finally {
            setLoading(false);
        }
    }

    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <form onSubmit={handleSubmit}>
                    <Stack gap="lg" align="center">
                        <Title order={2}>Sign in to EVMAuth</Title>
                        <Text c="dimmed" ta="center">
                            Enter your email to sign in or create an account.
                        </Text>

                        {error && (
                            <Alert color="red" w="100%">
                                {error}
                            </Alert>
                        )}

                        <TextInput
                            type="email"
                            placeholder="you@example.com"
                            value={email}
                            onChange={(e) => setEmail(e.currentTarget.value)}
                            required
                            w="100%"
                            size="md"
                        />

                        <Button
                            type="submit"
                            fullWidth
                            size="lg"
                            loading={loading}
                            disabled={!email}
                        >
                            Sign in
                        </Button>
                    </Stack>
                </form>
            </Paper>
        </Container>
    );
}
