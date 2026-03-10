'use client';

import { MantineProvider } from '@mantine/core';
import { Notifications } from '@mantine/notifications';
import type { ReactNode } from 'react';
import { theme } from '../theme';

import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';

interface ThemeProviderProps {
	children: ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
	return (
		<MantineProvider theme={theme} defaultColorScheme="auto">
			<Notifications position="top-right" />
			{children}
		</MantineProvider>
	);
}
