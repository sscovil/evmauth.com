'use client';

import { api } from '@/lib/api-client';
import { useApp } from '@/lib/hooks';
import {
    Alert,
    Button,
    Card,
    Code,
    Group,
    Skeleton,
    Stack,
    TagsInput,
    Text,
    TextInput,
    Title,
} from '@mantine/core';
import { useParams, useRouter } from 'next/navigation';
import { type ReactElement, useEffect, useState } from 'react';
import { useSWRConfig } from 'swr';

export default function AppDetailPage(): ReactElement {
    const { orgId, appId } = useParams<{ orgId: string; appId: string }>();
    const router = useRouter();
    const { data: app, error, isLoading } = useApp(orgId, appId);
    const { mutate } = useSWRConfig();

    const [editing, setEditing] = useState(false);
    const [name, setName] = useState('');
    const [callbackUrls, setCallbackUrls] = useState<string[]>([]);
    const [saving, setSaving] = useState(false);
    const [saveError, setSaveError] = useState('');
    const [deleting, setDeleting] = useState(false);

    useEffect(() => {
        if (app) {
            setName(app.name);
            setCallbackUrls(app.callback_urls);
        }
    }, [app]);

    async function handleSave(): Promise<void> {
        setSaving(true);
        setSaveError('');
        try {
            await api.patch(`/registry/orgs/${orgId}/apps/${appId}`, {
                name,
                callback_urls: callbackUrls,
            });
            await mutate(`/api/proxy/registry/orgs/${orgId}/apps/${appId}`);
            setEditing(false);
        } catch (e) {
            setSaveError(e instanceof Error ? e.message : 'Failed to save');
        } finally {
            setSaving(false);
        }
    }

    async function handleDelete(): Promise<void> {
        setDeleting(true);
        try {
            await api.delete(`/registry/orgs/${orgId}/apps/${appId}`);
            await mutate(`/api/proxy/registry/orgs/${orgId}/apps`);
            router.push(`/dashboard/${orgId}/apps`);
        } catch (e) {
            setSaveError(e instanceof Error ? e.message : 'Failed to delete');
            setDeleting(false);
        }
    }

    if (isLoading) {
        return (
            <Stack>
                <Skeleton height={30} width="40%" />
                <Skeleton height={200} />
            </Stack>
        );
    }

    if (error || !app) {
        return (
            <Alert color="red" title="Error">
                {error?.message ?? 'App not found'}
            </Alert>
        );
    }

    return (
        <Stack>
            <Group justify="space-between">
                <Title order={3}>{app.name}</Title>
                <Group>
                    {!editing && (
                        <Button variant="light" onClick={() => setEditing(true)}>
                            Edit
                        </Button>
                    )}
                    <Button variant="light" color="red" onClick={handleDelete} loading={deleting}>
                        Delete
                    </Button>
                </Group>
            </Group>

            <Card shadow="sm" padding="lg" radius="md" withBorder>
                <Stack>
                    <div>
                        <Text size="sm" fw={500} mb={4}>
                            Client ID
                        </Text>
                        <Code>{app.client_id}</Code>
                    </div>

                    {editing ? (
                        <>
                            <TextInput
                                label="Name"
                                value={name}
                                onChange={(e) => setName(e.currentTarget.value)}
                            />
                            <TagsInput
                                label="Callback URLs"
                                placeholder="https://example.com/callback"
                                value={callbackUrls}
                                onChange={setCallbackUrls}
                            />
                            {saveError && (
                                <Alert color="red" variant="light">
                                    {saveError}
                                </Alert>
                            )}
                            <Group>
                                <Button onClick={handleSave} loading={saving}>
                                    Save
                                </Button>
                                <Button variant="subtle" onClick={() => setEditing(false)}>
                                    Cancel
                                </Button>
                            </Group>
                        </>
                    ) : (
                        <>
                            <div>
                                <Text size="sm" fw={500} mb={4}>
                                    Callback URLs
                                </Text>
                                {app.callback_urls.length > 0 ? (
                                    app.callback_urls.map((url) => (
                                        <Code key={url} display="block" mb={2}>
                                            {url}
                                        </Code>
                                    ))
                                ) : (
                                    <Text size="sm" c="dimmed">
                                        None configured
                                    </Text>
                                )}
                            </div>
                            <div>
                                <Text size="sm" fw={500} mb={4}>
                                    Relevant Token IDs
                                </Text>
                                {app.relevant_token_ids.length > 0 ? (
                                    <Code>{app.relevant_token_ids.join(', ')}</Code>
                                ) : (
                                    <Text size="sm" c="dimmed">
                                        None configured
                                    </Text>
                                )}
                            </div>
                        </>
                    )}
                </Stack>
            </Card>
        </Stack>
    );
}
