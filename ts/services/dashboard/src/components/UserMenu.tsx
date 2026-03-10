'use client';

import { useMe } from '@/lib/hooks';
import { Button, Menu, Text, UnstyledButton } from '@mantine/core';
import { useRouter } from 'next/navigation';

export function UserMenu() {
    const router = useRouter();
    const { data: me } = useMe();

    async function handleLogout() {
        await fetch('/api/auth/logout', { method: 'POST', credentials: 'include' });
        router.push('/auth/login');
    }

    if (!me) {
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
