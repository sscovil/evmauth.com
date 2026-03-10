FROM node:22-alpine AS builder
RUN corepack enable && corepack prepare pnpm@9.15.0 --activate
WORKDIR /app
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY packages/ ./packages/
COPY services/dashboard/package.json ./services/dashboard/
RUN pnpm install --frozen-lockfile
COPY services/dashboard/ ./services/dashboard/
RUN pnpm --filter dashboard run build

FROM node:22-alpine
WORKDIR /app
COPY --from=builder /app/services/dashboard/.next/standalone ./
COPY --from=builder /app/services/dashboard/.next/static ./.next/static
COPY --from=builder /app/services/dashboard/public ./public
EXPOSE 3000
CMD ["node", "server.js"]
