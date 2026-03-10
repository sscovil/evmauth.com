'use client';

import { OrgCard } from '@/components/OrgCard';
import { useOrgs } from '@/lib/hooks';
import { Alert, SimpleGrid, Skeleton, Stack, Text } from '@mantine/core';

export function OrgList() {
    const { data, error, isLoading } = useOrgs();

    if (isLoading) {
        return (
            <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="md">
                <Skeleton height={100} radius="md" />
                <Skeleton height={100} radius="md" />
                <Skeleton height={100} radius="md" />
            </SimpleGrid>
        );
    }

    if (error) {
        return (
            <Alert color="red" title="Error loading organizations">
                {error.message}
            </Alert>
        );
    }

    const orgs = data?.edges?.map((edge) => edge.node) ?? [];

    if (orgs.length === 0) {
        return (
            <Stack align="center" py="xl">
                <Text c="dimmed">No organizations found.</Text>
            </Stack>
        );
    }

    return (
        <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="md">
            {orgs.map((org) => (
                <OrgCard key={org.id} org={org} />
            ))}
        </SimpleGrid>
    );
}
