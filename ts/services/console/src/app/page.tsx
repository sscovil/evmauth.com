import { Button, Container, Stack, Text, Title } from '@mantine/core';
import type { Metadata } from 'next';
import Link from 'next/link';
import type { ReactElement } from 'react';

export const metadata: Metadata = {
    title: 'EVMAuth',
    description: 'Authorization state management built on ERC-6909.',
};

export default function HomePage(): ReactElement {
    return (
        <Container size="sm" py="xl">
            <Stack align="center" gap="lg" mt={100}>
                <Title order={1}>EVMAuth</Title>
                <Text size="lg" c="dimmed" ta="center">
                    Authorization state management built on ERC-6909.
                </Text>
                <Button component={Link} href="/dashboard" size="lg">
                    Go to Dashboard
                </Button>
            </Stack>
        </Container>
    );
}
