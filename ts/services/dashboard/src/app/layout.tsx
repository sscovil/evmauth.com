import type { Metadata } from 'next';
import type { ReactNode } from 'react';
import { ThemeProvider } from '@evmauth/ui';

export const metadata: Metadata = {
	title: 'EVMAuth Dashboard',
	description: 'Manage your EVMAuth contracts, apps, and organization',
};

export default function RootLayout({ children }: { children: ReactNode }) {
	return (
		<html lang="en" suppressHydrationWarning>
			<body>
				<ThemeProvider>{children}</ThemeProvider>
			</body>
		</html>
	);
}
