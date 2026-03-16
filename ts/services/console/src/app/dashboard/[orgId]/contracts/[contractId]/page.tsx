'use client';

import { api } from '@/lib/api-client';
import { useContract, useRoleGrants } from '@/lib/hooks';
import type { RoleGrantResponse } from '@/types/api';
import {
    Alert,
    Badge,
    Button,
    Card,
    Code,
    Group,
    Modal,
    Select,
    Skeleton,
    Stack,
    Table,
    Text,
    Title,
} from '@mantine/core';
import { useParams } from 'next/navigation';
import { type ReactElement, useState } from 'react';
import { useSWRConfig } from 'swr';

const ROLES = [
    'DEFAULT_ADMIN_ROLE',
    'TOKEN_MANAGER_ROLE',
    'ACCESS_MANAGER_ROLE',
    'MINTER_ROLE',
    'BURNER_ROLE',
    'TREASURER_ROLE',
];

export default function ContractDetailPage(): ReactElement {
    const { orgId, contractId } = useParams<{ orgId: string; contractId: string }>();
    const { data: contract, error, isLoading } = useContract(orgId, contractId);
    const {
        data: roleGrants,
        error: rolesError,
        isLoading: rolesLoading,
    } = useRoleGrants(orgId, contractId);
    const { mutate } = useSWRConfig();

    const [grantOpen, setGrantOpen] = useState(false);
    const [role, setRole] = useState<string | null>(null);
    const [granting, setGranting] = useState(false);
    const [grantError, setGrantError] = useState('');
    const [revokingId, setRevokingId] = useState<string | null>(null);

    const rolesKey = `/api/proxy/registry/orgs/${orgId}/contracts/${contractId}/roles`;

    async function handleGrant(): Promise<void> {
        if (!role) return;
        setGranting(true);
        setGrantError('');
        try {
            await api.post(`/registry/orgs/${orgId}/contracts/${contractId}/roles`, { role });
            await mutate(rolesKey);
            setGrantOpen(false);
            setRole(null);
        } catch (e) {
            setGrantError(e instanceof Error ? e.message : 'Failed to grant role');
        } finally {
            setGranting(false);
        }
    }

    async function handleRevoke(grant: RoleGrantResponse): Promise<void> {
        setRevokingId(grant.id);
        try {
            await api.delete(`/registry/orgs/${orgId}/contracts/${contractId}/roles/${grant.id}`);
            await mutate(rolesKey);
        } catch (e) {
            setGrantError(e instanceof Error ? e.message : 'Failed to revoke role');
        } finally {
            setRevokingId(null);
        }
    }

    if (isLoading) {
        return (
            <Stack>
                <Skeleton height={30} width="40%" />
                <Skeleton height={200} />
            </Stack>
        );
    }

    if (error || !contract) {
        return (
            <Alert color="red" title="Error">
                {error?.message ?? 'Contract not found'}
            </Alert>
        );
    }

    return (
        <Stack>
            <Title order={3}>{contract.name}</Title>

            <Card shadow="sm" padding="lg" radius="md" withBorder>
                <Stack gap="sm">
                    <div>
                        <Text size="sm" fw={500} mb={4}>
                            Address
                        </Text>
                        <Code>{contract.address}</Code>
                    </div>
                    <Group>
                        <div>
                            <Text size="sm" fw={500} mb={4}>
                                Chain ID
                            </Text>
                            <Badge variant="light">{contract.chain_id}</Badge>
                        </div>
                        <div>
                            <Text size="sm" fw={500} mb={4}>
                                Beacon
                            </Text>
                            <Code style={{ fontSize: '0.75rem' }}>{contract.beacon_address}</Code>
                        </div>
                    </Group>
                    <div>
                        <Text size="sm" fw={500} mb={4}>
                            Deploy Transaction
                        </Text>
                        <Code style={{ fontSize: '0.75rem' }}>{contract.deploy_tx_hash}</Code>
                    </div>
                </Stack>
            </Card>

            <Group justify="space-between" mt="md">
                <Title order={4}>Role Grants</Title>
                <Button size="sm" onClick={() => setGrantOpen(true)}>
                    Grant Role
                </Button>
            </Group>

            {grantError && (
                <Alert color="red" variant="light">
                    {grantError}
                </Alert>
            )}

            {rolesLoading ? (
                <Skeleton height={100} />
            ) : rolesError ? (
                <Alert color="red" title="Error loading roles">
                    {rolesError.message}
                </Alert>
            ) : !roleGrants || roleGrants.length === 0 ? (
                <Text c="dimmed">No role grants yet.</Text>
            ) : (
                <Table striped highlightOnHover withTableBorder>
                    <Table.Thead>
                        <Table.Tr>
                            <Table.Th>Role</Table.Th>
                            <Table.Th>Status</Table.Th>
                            <Table.Th>Granted</Table.Th>
                            <Table.Th />
                        </Table.Tr>
                    </Table.Thead>
                    <Table.Tbody>
                        {roleGrants.map((grant) => (
                            <Table.Tr key={grant.id}>
                                <Table.Td>
                                    <Code>{grant.role}</Code>
                                </Table.Td>
                                <Table.Td>
                                    <Badge color={grant.active ? 'green' : 'gray'} variant="light">
                                        {grant.active ? 'Active' : 'Revoked'}
                                    </Badge>
                                </Table.Td>
                                <Table.Td>
                                    <Text size="sm">
                                        {new Date(grant.granted_at).toLocaleDateString()}
                                    </Text>
                                </Table.Td>
                                <Table.Td>
                                    {grant.active && (
                                        <Button
                                            size="xs"
                                            variant="light"
                                            color="red"
                                            loading={revokingId === grant.id}
                                            onClick={() => handleRevoke(grant)}
                                        >
                                            Revoke
                                        </Button>
                                    )}
                                </Table.Td>
                            </Table.Tr>
                        ))}
                    </Table.Tbody>
                </Table>
            )}

            <Modal opened={grantOpen} onClose={() => setGrantOpen(false)} title="Grant Role">
                <Stack>
                    <Select
                        label="Role"
                        placeholder="Select a role"
                        data={ROLES}
                        value={role}
                        onChange={setRole}
                        required
                    />
                    {grantError && (
                        <Alert color="red" variant="light">
                            {grantError}
                        </Alert>
                    )}
                    <Button onClick={handleGrant} loading={granting} disabled={!role}>
                        Grant
                    </Button>
                </Stack>
            </Modal>
        </Stack>
    );
}
