import { Container, Skeleton, Stack } from '@mantine/core';
import type { ReactElement } from 'react';

export default function DashboardLoading(): ReactElement {
    return (
        <Container size="sm" py="xl">
            <Stack gap="md">
                <Skeleton height={30} width="40%" />
                <Skeleton height={20} width="60%" />
                <Skeleton height={120} />
                <Skeleton height={120} />
                <Skeleton height={120} />
            </Stack>
        </Container>
    );
}
