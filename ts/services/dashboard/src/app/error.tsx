'use client';

import { Alert, Button, Container } from '@mantine/core';
import type { ReactElement } from 'react';

interface ErrorPageProps {
    error: Error & { digest?: string };
    reset: () => void;
}

export default function ErrorPage({ error, reset }: ErrorPageProps): ReactElement {
    return (
        <Container size="sm" py="xl">
            <Alert color="red" title="Something went wrong">
                {error.message || 'An unexpected error occurred.'}
            </Alert>
            <Button mt="md" onClick={reset}>
                Try again
            </Button>
        </Container>
    );
}
