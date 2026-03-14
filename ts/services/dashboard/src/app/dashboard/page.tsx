import { OrgList } from '@/components/OrgList';
import { Container, Text, Title } from '@mantine/core';
import type { ReactElement } from 'react';

export default function DashboardPage(): ReactElement {
    return (
        <Container>
            <Title order={2} mb="xs">
                Organizations
            </Title>
            <Text c="dimmed" mb="lg">
                Select an organization to manage its apps, contracts, and members.
            </Text>
            <OrgList />
        </Container>
    );
}
