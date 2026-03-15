'use client';

import { Alert, Button, Container } from '@mantine/core';
import type { ReactElement } from 'react';

interface LoginErrorProps {
    error: Error & { digest?: string };
    reset: () => void;
}

export default function LoginError({ error, reset }: LoginErrorProps): ReactElement {
    return (
        <Container size="sm" py="xl">
            <Alert color="red" title="Login error">
                {error.message || 'An unexpected error occurred.'}
            </Alert>
            <Button mt="md" onClick={reset}>
                Try again
            </Button>
        </Container>
    );
}
