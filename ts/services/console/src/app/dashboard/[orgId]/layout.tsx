'use client';

import { useOrg } from '@/lib/hooks';
import { Anchor, Group, Skeleton, Tabs, Text } from '@mantine/core';
import Link from 'next/link';
import { useParams, usePathname, useRouter } from 'next/navigation';
import type { ReactElement, ReactNode } from 'react';

interface OrgLayoutProps {
    children: ReactNode;
}

export default function OrgLayout({ children }: OrgLayoutProps): ReactElement {
    const { orgId } = useParams<{ orgId: string }>();
    const pathname = usePathname();
    const router = useRouter();
    const { data: org, isLoading } = useOrg(orgId);

    const activeTab = pathname.includes('/contracts') ? 'contracts' : 'apps';

    function handleTabChange(value: string | null): void {
        if (value) {
            router.push(`/dashboard/${orgId}/${value}`);
        }
    }

    return (
        <>
            <Group mb="md">
                <Anchor component={Link} href="/dashboard" size="sm">
                    Organizations
                </Anchor>
                <Text size="sm" c="dimmed">
                    /
                </Text>
                {isLoading ? (
                    <Skeleton height={16} width={120} />
                ) : (
                    <Text size="sm" fw={500}>
                        {org?.display_name}
                    </Text>
                )}
            </Group>

            <Tabs value={activeTab} onChange={handleTabChange} mb="lg">
                <Tabs.List>
                    <Tabs.Tab value="apps">Apps</Tabs.Tab>
                    <Tabs.Tab value="contracts">Contracts</Tabs.Tab>
                </Tabs.List>
            </Tabs>

            {children}
        </>
    );
}
