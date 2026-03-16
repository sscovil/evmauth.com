'use client';

import {
    Alert,
    Button,
    Card,
    Code,
    Container,
    Loader,
    Stack,
    Text,
    TextInput,
    Title,
} from '@mantine/core';
import { getWebAuthnAttestation } from '@turnkey/sdk-browser';
import { type ReactElement, useEffect, useState } from 'react';

interface AppWallet {
    id: string;
    entity_id: string;
    app_registration_id: string;
    wallet_address: string;
    turnkey_account_id: string;
    created_at: string;
    updated_at: string;
}

export default function WalletSelfServicePage(): ReactElement {
    const [wallets, setWallets] = useState<AppWallet[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState('');

    const [passkeyName, setPasskeyName] = useState('');
    const [passkeyLoading, setPasskeyLoading] = useState(false);
    const [passkeySuccess, setPasskeySuccess] = useState('');
    const [passkeyError, setPasskeyError] = useState('');

    useEffect(() => {
        fetch('/api/proxy/wallets/me/wallets', { credentials: 'include' })
            .then(async (resp) => {
                if (!resp.ok) {
                    throw new Error(`Failed to load wallets (${resp.status})`);
                }
                const data = (await resp.json()) as AppWallet[];
                setWallets(data);
            })
            .catch((e: Error) => {
                setError(e.message);
            })
            .finally(() => {
                setLoading(false);
            });
    }, []);

    const handleAddPasskey = async (): Promise<void> => {
        setPasskeyError('');
        setPasskeySuccess('');
        setPasskeyLoading(true);

        try {
            const challenge = crypto.randomUUID();

            const attestation = await getWebAuthnAttestation({
                publicKey: {
                    rp: { id: window.location.hostname, name: 'EVMAuth' },
                    user: {
                        id: new TextEncoder().encode(challenge),
                        name: passkeyName || 'Backup Passkey',
                        displayName: passkeyName || 'Backup Passkey',
                    },
                    challenge: new TextEncoder().encode(challenge),
                    pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
                    authenticatorSelection: {
                        residentKey: 'preferred',
                        userVerification: 'required',
                    },
                },
            });

            const resp = await fetch('/api/proxy/auth/me/authenticators', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                    authenticator_name: passkeyName || 'Backup Passkey',
                    challenge,
                    attestation,
                }),
            });

            if (!resp.ok) {
                const body = await resp.text();
                throw new Error(`Failed to register passkey (${resp.status}): ${body}`);
            }

            setPasskeySuccess('Backup passkey registered successfully.');
            setPasskeyName('');
        } catch (e: unknown) {
            const message = e instanceof Error ? e.message : 'Unknown error';
            setPasskeyError(message);
        } finally {
            setPasskeyLoading(false);
        }
    };

    if (loading) {
        return (
            <Container size="sm" py="xl">
                <Stack align="center" mt={100}>
                    <Loader />
                    <Text c="dimmed">Loading your wallets...</Text>
                </Stack>
            </Container>
        );
    }

    if (error) {
        return (
            <Container size="sm" py="xl">
                <Alert color="red" title="Error" mt={100}>
                    {error}
                </Alert>
            </Container>
        );
    }

    return (
        <Container size="sm" py="xl">
            <Title order={2} mb="xs">
                Your Wallets
            </Title>
            <Text c="dimmed" mb="lg">
                Wallet addresses associated with your EVMAuth account across different apps.
            </Text>

            {wallets.length === 0 ? (
                <Text c="dimmed">
                    No wallets found. Wallets are created when you sign in to apps.
                </Text>
            ) : (
                <Stack>
                    {wallets.map((wallet) => (
                        <Card key={wallet.id} shadow="sm" padding="lg" radius="md" withBorder>
                            <Stack gap="xs">
                                <div>
                                    <Text size="sm" fw={500} mb={4}>
                                        Wallet Address
                                    </Text>
                                    <Code>{wallet.wallet_address}</Code>
                                </div>
                                <div>
                                    <Text size="sm" fw={500} mb={4}>
                                        App Registration
                                    </Text>
                                    <Code style={{ fontSize: '0.75rem' }}>
                                        {wallet.app_registration_id}
                                    </Code>
                                </div>
                                <Text size="xs" c="dimmed">
                                    Created {new Date(wallet.created_at).toLocaleDateString()}
                                </Text>
                            </Stack>
                        </Card>
                    ))}
                </Stack>
            )}

            <Title order={3} mt="xl" mb="xs">
                Backup Passkey
            </Title>
            <Text c="dimmed" mb="md">
                Register a passkey as a backup authenticator for your account.
            </Text>

            {passkeySuccess && (
                <Alert color="green" title="Success" mb="md">
                    {passkeySuccess}
                </Alert>
            )}
            {passkeyError && (
                <Alert color="red" title="Error" mb="md">
                    {passkeyError}
                </Alert>
            )}

            <Stack gap="sm">
                <TextInput
                    label="Passkey Name"
                    placeholder="e.g. MacBook Touch ID"
                    value={passkeyName}
                    onChange={(e) => setPasskeyName(e.currentTarget.value)}
                />
                <Button onClick={handleAddPasskey} loading={passkeyLoading} variant="outline">
                    Add Backup Passkey
                </Button>
            </Stack>
        </Container>
    );
}
