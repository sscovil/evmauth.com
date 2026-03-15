'use client';

import type { OrgResponse } from '@/types/api';
import { Badge, Card, Group, Text } from '@mantine/core';
import Link from 'next/link';
import type { ReactElement } from 'react';

const visibilityColors: Record<OrgResponse['visibility'], string> = {
    personal: 'blue',
    private: 'gray',
    public: 'green',
};

interface OrgCardProps {
    org: OrgResponse;
}

export function OrgCard({ org }: OrgCardProps): ReactElement {
    return (
        <Card
            shadow="sm"
            padding="lg"
            radius="md"
            withBorder
            component={Link}
            href={`/dashboard/${org.id}`}
            style={{ textDecoration: 'none' }}
        >
            <Group justify="space-between" mb="xs">
                <Text fw={500}>{org.display_name}</Text>
                <Badge color={visibilityColors[org.visibility]} variant="light">
                    {org.visibility}
                </Badge>
            </Group>

            {org.description && (
                <Text size="sm" c="dimmed">
                    {org.description}
                </Text>
            )}
        </Card>
    );
}
