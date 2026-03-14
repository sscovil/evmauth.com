import type { Metadata } from 'next';
import type { ReactElement, ReactNode } from 'react';

export const metadata: Metadata = {
    title: 'Log in - EVMAuth',
    description: 'Log in to the EVMAuth dashboard.',
};

interface LoginLayoutProps {
    children: ReactNode;
}

export default function LoginLayout({ children }: LoginLayoutProps): ReactElement {
    return <>{children}</>;
}
