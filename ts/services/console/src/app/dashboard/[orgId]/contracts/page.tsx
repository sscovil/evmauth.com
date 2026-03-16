'use client';

import { api } from '@/lib/api-client';
import { useApps, useContracts } from '@/lib/hooks';
import type { AppRegistrationResponse, ContractResponse, Edge } from '@/types/api';
import {
    Alert,
    Badge,
    Button,
    Card,
    Code,
    Group,
    Modal,
    Select,
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

export default function ContractsPage(): ReactElement {
    const { orgId } = useParams<{ orgId: string }>();
    const { data, error, isLoading } = useContracts(orgId);
    const { data: appsData } = useApps(orgId);
    const { mutate } = useSWRConfig();
    const [deployOpen, setDeployOpen] = useState(false);
    const [name, setName] = useState('');
    const [appId, setAppId] = useState<string | null>(null);
    const [deploying, setDeploying] = useState(false);
    const [deployError, setDeployError] = useState('');

    const apps = appsData?.edges?.map((e: Edge<AppRegistrationResponse>) => e.node) ?? [];

    async function handleDeploy(): Promise<void> {
        if (!appId) return;
        setDeploying(true);
        setDeployError('');
        try {
            await api.post(`/registry/orgs/${orgId}/contracts`, {
                name,
                app_registration_id: appId,
            });
            await mutate(`/api/proxy/registry/orgs/${orgId}/contracts`);
            setDeployOpen(false);
            setName('');
            setAppId(null);
        } catch (e) {
            setDeployError(e instanceof Error ? e.message : 'Deployment failed');
        } finally {
            setDeploying(false);
        }
    }

    if (isLoading) {
        return (
            <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
                <Skeleton height={120} radius="md" />
                <Skeleton height={120} radius="md" />
            </SimpleGrid>
        );
    }

    if (error) {
        return (
            <Alert color="red" title="Error loading contracts">
                {error.message}
            </Alert>
        );
    }

    const contracts = data?.edges?.map((edge: Edge<ContractResponse>) => edge.node) ?? [];

    return (
        <>
            <Group justify="space-between" mb="md">
                <Title order={3}>Contracts</Title>
                <Button onClick={() => setDeployOpen(true)}>Deploy Contract</Button>
            </Group>

            {contracts.length === 0 ? (
                <Text c="dimmed">No contracts deployed yet.</Text>
            ) : (
                <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
                    {contracts.map((contract) => (
                        <Card
                            key={contract.id}
                            shadow="sm"
                            padding="lg"
                            radius="md"
                            withBorder
                            component={Link}
                            href={`/dashboard/${orgId}/contracts/${contract.id}`}
                            style={{ textDecoration: 'none' }}
                        >
                            <Group justify="space-between" mb="xs">
                                <Text fw={500}>{contract.name}</Text>
                                <Badge variant="light" size="sm">
                                    Chain {contract.chain_id}
                                </Badge>
                            </Group>
                            <Code display="block" style={{ fontSize: '0.75rem' }}>
                                {contract.address}
                            </Code>
                        </Card>
                    ))}
                </SimpleGrid>
            )}

            <Modal opened={deployOpen} onClose={() => setDeployOpen(false)} title="Deploy Contract">
                <Stack>
                    <TextInput
                        label="Contract Name"
                        placeholder="My EVMAuth Contract"
                        value={name}
                        onChange={(e) => setName(e.currentTarget.value)}
                        required
                    />
                    <Select
                        label="App Registration"
                        placeholder="Select an app"
                        data={apps.map((a) => ({ value: a.id, label: a.name }))}
                        value={appId}
                        onChange={setAppId}
                        required
                    />
                    {deployError && (
                        <Alert color="red" variant="light">
                            {deployError}
                        </Alert>
                    )}
                    <Button
                        onClick={handleDeploy}
                        loading={deploying}
                        disabled={!name.trim() || !appId}
                    >
                        Deploy
                    </Button>
                </Stack>
            </Modal>
        </>
    );
}
