FROM node:22-alpine AS base
RUN corepack enable && corepack prepare pnpm@9.15.0 --activate
WORKDIR /workspace

# Dev stage -- used by Tilt for hot reload
FROM base AS dev
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml* ./
COPY packages/ ./packages/
ARG SERVICE
COPY services/${SERVICE}/package.json ./services/${SERVICE}/
RUN pnpm install || true
CMD ["sh", "-c", "pnpm --filter ${SERVICE} dev"]

# Builder stage -- production build
FROM base AS builder
ARG SERVICE
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY packages/ ./packages/
COPY services/${SERVICE}/package.json ./services/${SERVICE}/
RUN pnpm install --frozen-lockfile
COPY services/${SERVICE}/ ./services/${SERVICE}/
RUN pnpm --filter ${SERVICE} run build

# Runtime stage -- standalone Next.js output
FROM node:22-alpine AS runtime
WORKDIR /app
ARG SERVICE
COPY --from=builder /workspace/services/${SERVICE}/.next/standalone ./
COPY --from=builder /workspace/services/${SERVICE}/.next/static ./.next/static
COPY --from=builder /workspace/services/${SERVICE}/public ./public
EXPOSE 3000
CMD ["node", "server.js"]
