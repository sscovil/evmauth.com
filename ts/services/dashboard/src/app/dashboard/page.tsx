import { Container, Text, Title } from '@mantine/core';

export default function DashboardPage() {
    return (
        <Container>
            <Title order={2} mb="md">
                Organizations
            </Title>
            <Text c="dimmed">
                Select an organization to manage its apps, contracts, and members.
            </Text>
        </Container>
    );
}
