'use client';

import { authenticate } from '@/lib/api-client';
import { Button, Container, Paper, Stack, Text, TextInput, Title } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useRouter } from 'next/navigation';
import { type ReactElement, useState } from 'react';
import { mutate } from 'swr';

export default function LoginPage(): ReactElement {
    const router = useRouter();
    const [loading, setLoading] = useState(false);

    const form = useForm({
        initialValues: { email: '' },
        validate: {
            email: (value: string) =>
                /^\S+@\S+\.\S+$/.test(value) ? null : 'Invalid email address',
        },
    });

    async function handleSubmit(values: { email: string }): Promise<void> {
        setLoading(true);
        form.clearErrors();
        try {
            await authenticate(values.email);
            await mutate('/api/auth/me');
            await mutate('/api/proxy/auth/orgs');
            router.push('/dashboard');
        } catch (err) {
            form.setFieldError(
                'email',
                err instanceof Error ? err.message : 'Authentication failed',
            );
        } finally {
            setLoading(false);
        }
    }

    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <form onSubmit={form.onSubmit(handleSubmit)}>
                    <Stack gap="lg" align="center">
                        <Title order={2}>Sign in to EVMAuth</Title>
                        <Text c="dimmed" ta="center">
                            Enter your email to sign in or create an account.
                        </Text>

                        <TextInput
                            type="email"
                            placeholder="you@example.com"
                            required
                            w="100%"
                            size="md"
                            {...form.getInputProps('email')}
                        />

                        <Button
                            type="submit"
                            fullWidth
                            size="lg"
                            loading={loading}
                            disabled={!form.values.email}
                        >
                            Sign in
                        </Button>
                    </Stack>
                </form>
            </Paper>
        </Container>
    );
}
