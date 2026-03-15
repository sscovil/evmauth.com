'use client';

import { UserMenu } from '@/components/UserMenu';
import { AppShell, Group, NavLink, Text } from '@mantine/core';
import Link from 'next/link';
import type { ReactElement, ReactNode } from 'react';

interface DashboardLayoutProps {
    children: ReactNode;
}

export default function DashboardLayout({ children }: DashboardLayoutProps): ReactElement {
    return (
        <AppShell header={{ height: 60 }} navbar={{ width: 250, breakpoint: 'sm' }} padding="md">
            <AppShell.Header>
                <Group h="100%" px="md" justify="space-between">
                    <Text fw={700} size="lg">
                        EVMAuth
                    </Text>
                    <UserMenu />
                </Group>
            </AppShell.Header>

            <AppShell.Navbar p="md">
                <NavLink label="Organizations" href="/dashboard" component={Link} />
            </AppShell.Navbar>

            <AppShell.Main>{children}</AppShell.Main>
        </AppShell>
    );
}
