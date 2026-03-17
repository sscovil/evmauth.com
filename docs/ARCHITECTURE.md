# EVMAuth Platform Architecture

This document explains how the EVMAuth managed service platform works, end to end.
It is written for engineers who will be reading, modifying, or extending the codebase.

## Mental Model

EVMAuth has two halves:

1. **Turnkey = authentication (identity).** Every person and organization gets a Turnkey
   sub-org with an HD wallet. The wallet address is the on-chain identity. Passkeys
   (WebAuthn) are the only credential type -- no passwords, no OAuth tokens.

2. **EVMAuth contracts = authorization (permissions).** Token balances on ERC-6909
   contracts represent what an identity is allowed to do. The platform never stores
   authorization state in a database. It reads it from the chain at query time.

The platform itself uses the same model: deployers hold a capability token
(token ID 1 on the platform contract) to access the API. There are no API keys or
client secrets anywhere in the system.

## Actors

**Beacon owner** -- EVMAuth's highest-privilege wallet. Owns the UpgradeableBeacon
contract (can upgrade logic for all proxies). Holds admin roles on the platform
contract. Rarely used.

**Platform operator** -- EVMAuth's operational wallet. Deploys BeaconProxy contracts,
mints and burns capability tokens. Used by the wallets service for routine operations.

**Deployer** -- An organization that registers an app and gets an EVMAuth proxy contract
deployed for it. The org's HD wallet (index 0) receives all roles on its contract at
initialization. The org owner manages roles and token types through the console.

**End user** -- A person who is a customer of a deployer's app. Gets a per-app derived
wallet address. The deployer mints tokens to this address to grant access.

**Delegate** -- An address granted operator rights by a wallet owner via ERC-6909
`setOperator`. An operator can transfer tokens on behalf of the owner. In the context
of the `/accounts` query, a delegate can query a principal's token balances -- the
endpoint verifies the delegation via `isOperator(owner, delegate)` on-chain.

## Services

The backend is a set of Rust/Axum microservices. Each service owns its own PostgreSQL
schema and communicates with other services via internal HTTP APIs (feature-gated
`/internal/*` routes). A gateway service proxies all external traffic by path prefix.

### Auth Service

Owns the `auth` schema: people, organizations, org memberships.

Key responsibilities:
- **Deployer signup**: accepts a passkey attestation (from `@turnkey/sdk-browser`),
  creates a person record, provisions a Turnkey sub-org + HD wallet via the wallets
  service, creates a personal workspace org with its own wallet, mints a capability
  token to the org wallet, and issues a session JWT.
- **Deployer login**: issues a challenge nonce (stored in Redis with 60s TTL), the
  client signs it with their wallet key, the backend recovers the signer via
  `ecrecover`, looks up the person by wallet address, and issues a session JWT.
- **End-user onboarding**: validates the `client_id` against the registry service,
  finds or creates the person + wallet, ensures an app-specific derived wallet exists,
  and returns the wallet address. No JWT is issued -- the deployer's app is responsible
  for its own session management.
- **Session middleware**: validates the session JWT (from cookie or Authorization header)
  and injects `AuthenticatedPerson { person_id, wallet_address }` into request
  extensions for protected routes.

The session JWT is RS256 with claims: `iss`, `sub` (person ID), `type: "session"`,
`wallet` (wallet address), `exp`, `iat`. It is set as an `HttpOnly; Secure;
SameSite=Strict` cookie with an 8-hour lifetime.

### Wallets Service

Owns the `wallets` schema: entity wallets, entity app wallets.

The only service that communicates with the Turnkey API. Uses the official Turnkey Rust
SDK (`turnkey_client` + `turnkey_api_key_stamper`) with P-256 ECDSA request signing.

Key responsibilities:
- Create Turnkey sub-orgs with HD wallets for people and orgs
- Derive per-app wallet accounts from an entity's HD wallet
- Sign raw payloads and broadcast transactions via Turnkey delegated accounts
- Add passkey authenticators to existing sub-orgs

Each entity (person or org) has one HD wallet. Account index 0 is the entity's platform
identity. Additional accounts are derived per app registration and stored in
`entity_app_wallets`.

For orgs, a delegated account is created with a P-256 API key scoped by Turnkey policy
to `ACTIVITY_TYPE_SIGN_RAW_PAYLOAD` only. This allows the platform to sign transactions
on behalf of the org without the org owner's passkey.

### Registry Service

Owns the `registry` schema: app registrations, contracts, role grants.

Key responsibilities:
- CRUD for app registrations (each gets a random `client_id`)
- Deploy EVMAuth BeaconProxy contracts via the wallets service
- Grant and revoke roles on deployed contracts
- Serve the `/accounts/{address}` authorization query endpoint

The `/accounts` endpoint is protected by ERC-712 middleware. Callers must include
signed request headers (`X-Client-Id`, `X-Signer`, `X-Signature`, `X-Nonce`,
`X-Contract`). The middleware verifies the signature, checks nonce freshness (30s
window), and confirms the signer holds the API access capability token on the platform
contract. The handler then queries on-chain token balances for the requested address
and returns them.

### Gateway

Routes external traffic to internal services by path prefix (e.g., `/auth/*` to the
auth service, `/wallets/*` to the wallets service). Resolves service URLs via the
`service-discovery` crate based on the deployment environment.

### Other Services

- **Docs** -- aggregates OpenAPI specs from all services into a unified Swagger UI.
- **Assets** -- file upload management with S3/MinIO backend.
- **Analytics** -- (planned) API request logging and contract event indexing.
- **DB** -- migration runner. All migrations live here and run on startup.

## Shared Crates

- **evm** -- Alloy-based EVM interaction. Read-only HTTP provider, typed EVMAuth6909
  contract bindings (`balanceOf`, `isOperator`, `mint`, etc.), calldata encoding, and
  signature verification (`recover_signer` for EIP-191, `verify_accounts_query` for
  ERC-712). Does not sign transactions -- only verifies signatures and encodes calldata.
- **types** -- transparent newtypes with Serde + SQLx support: `ChecksumAddress`,
  `TxHash`, `TurnkeySubOrgId`, `ClientId`.
- **pagination** -- cursor-based pagination following the Relay GraphQL Connections spec.
- **postgres** -- `PgPool` creation from a `PGConfig` struct.
- **redis-client** -- Redis `ConnectionManager` creation.
- **service-discovery** -- builds service URLs based on environment.

## Frontend

The console is a Next.js application at `ts/services/console/`. It serves both the
deployer dashboard and the hosted end-user onboarding pages.

### Authentication Flow

The login page handles everything client-side via WebAuthn:

- **Signup**: collects display name + email, calls `getWebAuthnAttestation()` from
  `@turnkey/sdk-browser` to create a passkey, sends the attestation to a Next.js API
  route which forwards it to the Rust backend.
- **Login**: fetches a challenge nonce from the backend, triggers
  `navigator.credentials.get()` to prompt the passkey, sends the signed assertion
  to a Next.js API route which forwards it to the Rust backend.

The Next.js API routes (`/api/auth/*`) act as a server-side intermediary: they call the
Rust backend, validate responses with Zod schemas, and manage an `iron-session` cookie
(`evmauth-console`) that stores `{ personId, email, displayName }`. The backend JWT is
never exposed to the browser.

### API Proxy

All dashboard data fetching goes through `/api/proxy/[...path]`, a catch-all Next.js
route that reads the iron-session, rejects unauthenticated requests, and forwards to the
Rust gateway with an `X-Person-Id` header. Raw browser cookies are never forwarded.

### Data Fetching

SWR hooks in `lib/hooks.ts` (`useMe`, `useOrgs`, `useApps`, etc.) wrap the API client
for all dashboard data. `mutate` is called after write operations to revalidate affected
cache keys.

## Contract Architecture

The platform uses the beacon proxy pattern. One `UpgradeableBeacon` contract stores the
address of the current EVMAuth6909 implementation. Every deployer gets a `BeaconProxy`
that delegates all calls to the beacon's implementation. Upgrading the beacon upgrades
all proxies atomically.

At proxy deployment time, `initialize()` grants `DEFAULT_ADMIN_ROLE` and all operational
roles (TOKEN_MANAGER, ACCESS_MANAGER, TREASURER, MINTER, BURNER) to the deployer org's
HD wallet address (index 0). The platform operator wallet deploys the proxy and pays gas,
but has no admin privileges on it.

The platform itself has its own EVMAuth proxy contract. Token ID 1 on this contract
represents API access. When a deployer org is created, a token is minted to the org's
wallet. The `/accounts` endpoint checks for this token before serving queries.

## Key Design Decisions

**No secrets anywhere.** There are no API keys, client secrets, or passwords in the
system. Deployer identity is a wallet address verified by signature. End-user identity
is a wallet address provisioned by Turnkey. API access is an on-chain token balance.

**Signing only happens in Turnkey.** The evm crate and auth service only verify
signatures (ecrecover). All transaction signing goes through the wallets service, which
delegates to Turnkey's TEE infrastructure.

**No end-user JWTs.** The platform does not issue tokens to deployer apps. End users
onboard via passkey and receive a wallet address. At runtime, the deployer's backend
queries `GET /accounts/{address}` with an ERC-712 signed request to check on-chain
balances. This eliminates token expiry, refresh flows, and revocation propagation delays.

**Schema-per-service isolation.** Each service owns its database schema. Cross-service
references use UUIDs without foreign key constraints. Referential integrity is maintained
via internal API calls.

**Compile-time checked queries.** All SQL uses `sqlx::query_as!` macros with an offline
query cache (`.sqlx/` directory). After adding or modifying queries, run
`tilt trigger sqlx-prepare` to regenerate the cache.

## Local Development

Infrastructure (PostgreSQL, Redis, MinIO, Anvil) runs via Docker Compose. Services run
via Tilt, which auto-discovers them by scanning for `service.json` files. Rust services
use `cargo watch` for hot reload; the Next.js console uses `pnpm dev`.

Contract deployment in local dev goes through three Tilt actions: `fund-wallets` (sends
ETH from Anvil to the beacon owner and platform operator addresses), `deploy-beacon`
(deploys the implementation contract), and `deploy-platform` (deploys the platform proxy).
All signing goes through Turnkey even in local dev -- Anvil private keys are only used
for the initial funding transfer.

Quality checks: `./check.sh` runs biome + tsc for TypeScript and fmt + clippy + test
for Rust.
