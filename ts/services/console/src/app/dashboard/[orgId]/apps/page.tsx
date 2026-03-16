'use client';

import { api } from '@/lib/api-client';
import { useApps } from '@/lib/hooks';
import type { AppRegistrationResponse, Edge } from '@/types/api';
import {
    Alert,
    Button,
    Card,
    Code,
    Group,
    Modal,
    SimpleGrid,
    Skeleton,
    Stack,
    Text,
    TextInput,
    Title,
} from '@mantine/core';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { type ReactElement, useState } from 'react';
import { useSWRConfig } from 'swr';

export default function AppsPage(): ReactElement {
    const { orgId } = useParams<{ orgId: string }>();
    const { data, error, isLoading } = useApps(orgId);
    const { mutate } = useSWRConfig();
    const [createOpen, setCreateOpen] = useState(false);
    const [name, setName] = useState('');
    const [creating, setCreating] = useState(false);
    const [createError, setCreateError] = useState('');

    async function handleCreate(): Promise<void> {
        setCreating(true);
        setCreateError('');
        try {
            await api.post(`/registry/orgs/${orgId}/apps`, { name });
            await mutate(`/api/proxy/registry/orgs/${orgId}/apps`);
            setCreateOpen(false);
            setName('');
        } catch (e) {
            setCreateError(e instanceof Error ? e.message : 'Failed to create app');
        } finally {
            setCreating(false);
        }
    }

    if (isLoading) {
        return (
            <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
                <Skeleton height={100} radius="md" />
                <Skeleton height={100} radius="md" />
            </SimpleGrid>
        );
    }

    if (error) {
        return (
            <Alert color="red" title="Error loading apps">
                {error.message}
            </Alert>
        );
    }

    const apps = data?.edges?.map((edge: Edge<AppRegistrationResponse>) => edge.node) ?? [];

    return (
        <>
            <Group justify="space-between" mb="md">
                <Title order={3}>App Registrations</Title>
                <Button onClick={() => setCreateOpen(true)}>Create App</Button>
            </Group>

            {apps.length === 0 ? (
                <Text c="dimmed">No app registrations yet.</Text>
            ) : (
                <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
                    {apps.map((app) => (
                        <Card
                            key={app.id}
                            shadow="sm"
                            padding="lg"
                            radius="md"
                            withBorder
                            component={Link}
                            href={`/dashboard/${orgId}/apps/${app.id}`}
                            style={{ textDecoration: 'none' }}
                        >
                            <Text fw={500} mb="xs">
                                {app.name}
                            </Text>
                            <Text size="xs" c="dimmed">
                                Client ID: <Code>{app.client_id}</Code>
                            </Text>
                        </Card>
                    ))}
                </SimpleGrid>
            )}

            <Modal
                opened={createOpen}
                onClose={() => setCreateOpen(false)}
                title="Create App Registration"
            >
                <Stack>
                    <TextInput
                        label="App Name"
                        placeholder="My Application"
                        value={name}
                        onChange={(e) => setName(e.currentTarget.value)}
                        required
                    />
                    {createError && (
                        <Alert color="red" variant="light">
                            {createError}
                        </Alert>
                    )}
                    <Button onClick={handleCreate} loading={creating} disabled={!name.trim()}>
                        Create
                    </Button>
                </Stack>
            </Modal>
        </>
    );
}
