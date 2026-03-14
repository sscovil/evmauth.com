'use client';

import { useMe } from '@/lib/hooks';
import { Button, Menu, Skeleton, Text, UnstyledButton } from '@mantine/core';
import { useRouter } from 'next/navigation';
import type { ReactElement } from 'react';
import { useSWRConfig } from 'swr';

export function UserMenu(): ReactElement | null {
    const router = useRouter();
    const { data: me, error, isLoading } = useMe();
    const { mutate } = useSWRConfig();

    async function handleLogout() {
        await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
        await mutate('/api/auth/me', undefined, { revalidate: false });
        router.push('/auth/login');
    }

    if (isLoading) {
        return <Skeleton width={80} height={20} />;
    }

    if (error || !me) {
        return null;
    }

    return (
        <Menu shadow="md" width={200}>
            <Menu.Target>
                <UnstyledButton>
                    <Text size="sm" fw={500}>
                        {me.displayName}
                    </Text>
                </UnstyledButton>
            </Menu.Target>

            <Menu.Dropdown>
                <Menu.Label>{me.email}</Menu.Label>
                <Menu.Divider />
                <Menu.Item>
                    <Button variant="subtle" size="compact-sm" fullWidth onClick={handleLogout}>
                        Sign out
                    </Button>
                </Menu.Item>
            </Menu.Dropdown>
        </Menu>
    );
}
