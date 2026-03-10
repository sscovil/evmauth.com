import { Button, Container, Paper, Stack, Text, Title } from '@mantine/core';

export default function LoginPage() {
    return (
        <Container size="xs" py="xl">
            <Paper shadow="md" p="xl" radius="md" mt={100}>
                <Stack gap="lg" align="center">
                    <Title order={2}>Sign in to EVMAuth</Title>
                    <Text c="dimmed" ta="center">
                        Use your passkey or continue with a social provider.
                    </Text>
                    <Button fullWidth size="lg">
                        Sign in with Passkey
                    </Button>
                    <Button fullWidth size="lg" variant="outline">
                        Continue with Google
                    </Button>
                </Stack>
            </Paper>
        </Container>
    );
}
