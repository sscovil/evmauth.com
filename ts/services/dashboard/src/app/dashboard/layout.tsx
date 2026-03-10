import { AppShell, Group, NavLink, Text } from '@mantine/core';
import type { ReactNode } from 'react';

export default function DashboardLayout({ children }: { children: ReactNode }) {
	return (
		<AppShell
			header={{ height: 60 }}
			navbar={{ width: 250, breakpoint: 'sm' }}
			padding="md"
		>
			<AppShell.Header>
				<Group h="100%" px="md" justify="space-between">
					<Text fw={700} size="lg">
						EVMAuth
					</Text>
				</Group>
			</AppShell.Header>

			<AppShell.Navbar p="md">
				<NavLink label="Organizations" href="/dashboard" />
			</AppShell.Navbar>

			<AppShell.Main>{children}</AppShell.Main>
		</AppShell>
	);
}
