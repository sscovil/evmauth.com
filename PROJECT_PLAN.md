# EVMAuth Managed Service Platform -- Project Plan

> **Audience:** Claude Code
> **Purpose:** End-to-end implementation guide for the EVMAuth managed service platform
> **Last updated:** 2026-03-15 (rev 12)

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Repository Structure](#2-repository-structure)
3. [Technology Stack](#3-technology-stack)
4. [Domain Model](#4-domain-model)
5. [Database Schema](#5-database-schema)
6. [Backend -- Rust / Axum Microservices](#6-backend--rust--axum-microservices)
7. [Frontend -- TypeScript / Next.js Workspace](#7-frontend--typescript--nextjs-workspace)
8. [Authentication & Identity Architecture](#8-authentication--identity-architecture)
9. [Contract Deployment & Management](#9-contract-deployment--management)
10. [Authorization Query API](#10-authorization-query-api)
11. [Local Development -- Tilt + Docker Compose](#11-local-development--tilt--docker-compose)
12. [Production Deployment -- Railway](#12-production-deployment--railway)
13. [Implementation Phases](#13-implementation-phases)
14. [Environment Variables Reference](#14-environment-variables-reference)

---

## 1. Project Overview

EVMAuth is an authorization state management system built on the ERC-6909 multi-token standard. This platform offers EVMAuth as a managed service: deployers register their apps, the platform deploys EVMAuth beacon proxy contracts on their behalf on the Radius Network, and the platform exposes an OIDC-like authentication flow plus a runtime authorization query API.

### Key Actors

| Actor | Description |
|---|---|
| **Beacon owner** | EVMAuth (you). Owns the Turnkey parent org and the beacon owner wallet. Controls the UpgradeableBeacon contract (can upgrade logic for all proxies). Holds `DEFAULT_ADMIN_ROLE`, `TOKEN_MANAGER_ROLE`, `ACCESS_MANAGER_ROLE`, and `TREASURER_ROLE` on the platform proxy. Rarely used, high-privilege. |
| **Platform operator** | EVMAuth (you). Owns the platform operator wallet. Deploys BeaconProxy contracts, pays gas, mints/burns capability tokens on the platform proxy (`MINTER_ROLE`, `BURNER_ROLE`). Frequently used by the wallets service for routine operations. |
| **Deployer** | A user or organization that registers an app and deploys an EVMAuth proxy contract. Administers their contract's EVMAuth roles via their org's Turnkey sub-org HD wallet (per-app derived accounts). The platform operator wallet deploys and owns the proxy at the EVM level. |
| **End user** | A customer of a deployer's app. Authenticates via the platform's hosted auth flow. Gets a personal Turnkey sub-org with an HD wallet; per-app accounts are derived from it. |
| **Delegate / Agent** | An address granted `operator` rights by an end user via ERC-6909 `setOperator`. Can call the authorization API on the principal's behalf. |

### Core Platform Responsibilities

- Manage Turnkey sub-org lifecycle for deployers and end users
- Deploy and upgrade EVMAuth beacon proxy contracts on Radius
- Use the platform's own EVMAuth contract for access control -- API access rights are ERC-6909 token holdings, not secrets
- Issue signed JWTs containing wallet address and contract reference (authentication only -- not authorization claims)
- Implement a full PKCE-based authorization code exchange for end-user auth (code -> JWT)
- Expose `GET /accounts/{address}` for runtime authorization queries, authenticated via ERC-712 request signing (reads live on-chain state)
- Provide a developer console for deployers to manage contracts, role grants, and app registrations
- Provide a hosted auth UI for end users (OAuth redirect flow)

---

## 2. Repository Structure

Monorepo with a Cargo workspace backend and a PNPM workspace frontend, both following the same microservices architecture. Each microservice owns its own database schema and domain. Services communicate via internal HTTP APIs through the gateway. Shared Rust libraries live in `rs/crates/`; shared TypeScript packages live in `ts/packages/`.

```
evmauth.com/
├── Tiltfile                       # Tilt orchestration (auto-discovers services)
├── docker-compose.yml             # Infrastructure: PostgreSQL, Redis, MinIO
├── docker/                        # Docker build scripts
├── check.sh                       # Quality checks: biome, tsc, fmt, clippy, test
├── .env.example
├── .env
├── CLAUDE.md                      # Development guidelines
├── rs/                            # Rust backend (Cargo workspace root)
│   ├── Cargo.toml                 # Workspace configuration
│   ├── Cargo.lock
│   ├── Dockerfile                 # Multi-stage production build
│   ├── OPENAPI.md                 # OpenAPI specification guide
│   ├── services/                  # Microservices (each is an independent binary)
│   │   ├── auth/                  # EXISTING: Authentication, people, orgs, memberships
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json       # Tilt metadata; variants: [int-auth]
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/mod.rs
│   │   │       ├── api/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── error.rs
│   │   │       │   ├── routes.rs
│   │   │       │   ├── openapi.rs
│   │   │       │   └── handlers/
│   │   │       │       ├── mod.rs
│   │   │       │       ├── health.rs
│   │   │       │       ├── people.rs
│   │   │       │       ├── orgs.rs
│   │   │       │       ├── org_members.rs
│   │   │       │       └── internal/
│   │   │       ├── domain/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── entity.rs
│   │   │       │   ├── person.rs
│   │   │       │   ├── org.rs
│   │   │       │   └── org_member.rs
│   │   │       ├── dto/
│   │   │       │   ├── request/
│   │   │       │   └── response/
│   │   │       └── repository/
│   │   │           ├── mod.rs
│   │   │           ├── error.rs
│   │   │           ├── person.rs
│   │   │           ├── org.rs
│   │   │           ├── org_member.rs
│   │   │           ├── entity.rs
│   │   │           ├── filter.rs
│   │   │           └── pagination.rs
│   │   ├── wallets/               # PARTIAL: Wallet lifecycle & Turnkey management
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/mod.rs
│   │   │       ├── api/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── error.rs
│   │   │       │   ├── routes.rs
│   │   │       │   ├── openapi.rs
│   │   │       │   └── handlers/
│   │   │       │       ├── mod.rs
│   │   │       │       ├── health.rs
│   │   │       │       ├── entity_wallets.rs
│   │   │       │       ├── entity_app_wallets.rs
│   │   │       │       └── internal/
│   │   │       ├── domain/
│   │   │       ├── dto/
│   │   │       └── repository/
│   │   ├── registry/              # TO BUILD: App registrations, contracts, accounts query
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/mod.rs
│   │   │       ├── api/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── error.rs
│   │   │       │   ├── routes.rs
│   │   │       │   ├── openapi.rs
│   │   │       │   └── handlers/
│   │   │       │       ├── mod.rs
│   │   │       │       ├── health.rs
│   │   │       │       ├── app_registrations.rs
│   │   │       │       ├── contracts.rs
│   │   │       │       ├── accounts.rs
│   │   │       │       └── internal/
│   │   │       ├── domain/
│   │   │       ├── dto/
│   │   │       ├── repository/
│   │   │       └── middleware/
│   │   │           └── erc712_auth.rs
│   │   ├── analytics/             # TO BUILD: Event indexing, request logging, metrics
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/mod.rs
│   │   │       ├── api/
│   │   │       ├── domain/
│   │   │       ├── dto/
│   │   │       └── repository/
│   │   ├── assets/                # EXISTING: File upload/management (S3/MinIO)
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/
│   │   │       ├── api/
│   │   │       ├── domain/
│   │   │       ├── dto/
│   │   │       ├── repository/
│   │   │       └── s3/
│   │   ├── gateway/               # EXISTING: API gateway (single entry point)
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── config/mod.rs
│   │   │       ├── proxy/
│   │   │       └── routes/mod.rs
│   │   ├── docs/                  # EXISTING: OpenAPI aggregation + Swagger UI
│   │   │   ├── Cargo.toml
│   │   │   ├── service.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── aggregator/
│   │   │       ├── config/
│   │   │       ├── routes/
│   │   │       └── static/
│   │   └── db/                    # EXISTING: Database migration runner
│   │       ├── Cargo.toml
│   │       ├── service.json
│   │       └── src/
│   │           ├── main.rs
│   │           ├── lib.rs
│   │           ├── config/
│   │           ├── migrations/    # All migrations (all schemas)
│   │           ├── seeds/
│   │           └── docs/
│   ├── tools/                     # Internal tools
│   │   └── cli/                   # evmauth-cli: contract deployment, dev funding
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── main.rs
│   │           └── commands/
│   │               ├── mod.rs
│   │               ├── deploy_beacon.rs
│   │               ├── deploy_platform.rs
│   │               └── fund.rs
│   └── crates/                    # Shared libraries
│       ├── pagination/            # EXISTING: Cursor-based pagination (Relay spec)
│       ├── pagination-macros/     # EXISTING: Proc macros for pagination
│       ├── postgres/              # EXISTING: PgPool creation + config
│       ├── redis-client/          # EXISTING: Redis ConnectionManager wrapper
│       ├── service-discovery/     # EXISTING: Service auto-discovery + health checks
│       └── evm/                   # EXISTING: Alloy-based EVM interaction
│           ├── Cargo.toml
│           └── src/
│               ├── lib.rs         # EvmError enum, re-exports (Address, Bytes, U256)
│               ├── client.rs      # EvmConfig, EvmClient (read-only Alloy HTTP provider)
│               └── evmauth.rs     # sol! bindings (balanceOf, isOperator, mint), encode_mint()
├── ts/                            # EXISTING: TypeScript frontend (PNPM workspace root)
│   ├── Dockerfile                 # Unified multi-stage build (dev/builder/runtime, SERVICE build arg)
│   ├── pnpm-workspace.yaml        # Workspace definition
│   ├── package.json               # Root scripts (e.g. pnpm check)
│   ├── biome.json                 # Shared Biome config
│   ├── tsconfig.json              # Base TypeScript config
│   ├── services/                  # Next.js apps (each is an independent app)
│   │   └── console/               # PARTIAL: Developer console + hosted auth UI
│   │       ├── package.json
│   │       ├── service.json       # Tilt metadata (ports, depends_on, etc.)
│   │       ├── next.config.ts
│   │       ├── tsconfig.json
│   │       └── src/
│   │           ├── middleware.ts          # Route protection (iron-session)
│   │           ├── app/
│   │           │   ├── layout.tsx
│   │           │   ├── page.tsx           # Landing page (with metadata)
│   │           │   ├── error.tsx          # Root error boundary
│   │           │   ├── dashboard/
│   │           │   │   ├── layout.tsx     # AppShell with UserMenu + sidebar
│   │           │   │   ├── page.tsx       # Org overview (OrgList)
│   │           │   │   ├── loading.tsx    # Skeleton loading state
│   │           │   │   ├── error.tsx      # Console error boundary
│   │           │   │   └── [orgSlug]/
│   │           │   │       ├── contracts/
│   │           │   │       ├── apps/
│   │           │   │       ├── members/
│   │           │   │       └── settings/
│   │           │   ├── auth/
│   │           │   │   ├── login/
│   │           │   │   │   ├── page.tsx   # Email login (Mantine useForm)
│   │           │   │   │   ├── layout.tsx # Login metadata
│   │           │   │   │   └── error.tsx  # Login error boundary
│   │           │   │   ├── callback/
│   │           │   │   └── end-user/
│   │           │   └── api/
│   │           │       ├── auth/          # login, signup, logout, me (Zod-validated)
│   │           │       └── proxy/         # Forwards to backend with session auth
│   │           ├── components/
│   │           │   ├── UserMenu.tsx       # Header dropdown (name, email, sign out)
│   │           │   ├── OrgCard.tsx        # Org card with visibility badge
│   │           │   └── OrgList.tsx        # Grid of OrgCards with loading/empty states
│   │           ├── lib/
│   │           │   ├── config.ts          # Validated env var access (single source of truth)
│   │           │   ├── schemas.ts         # Shared Zod schemas (TokenResponse, PersonResponse)
│   │           │   ├── api-client.ts      # Fetch-based API client + authenticate()
│   │           │   ├── session.ts         # iron-session config + SessionData type
│   │           │   └── hooks.ts           # SWR hooks (useMe, useOrgs)
│   │           └── types/
│   │               └── api.ts            # PersonResponse, OrgResponse, PaginatedResponse
│   └── packages/                  # Shared packages consumed by services
│       ├── ui/                    # Mantine theme, custom components
│       │   ├── package.json
│       │   ├── tsconfig.json
│       │   └── src/
│       │       ├── index.ts
│       │       ├── theme.ts
│       │       └── components/
│       └── tsconfig/              # Shared TypeScript configs
│           ├── package.json
│           ├── base.json
│           └── nextjs.json
└── contracts/                     # EXISTING: Solidity ABIs
    └── abis/
        └── EVMAuth6909.abi.json
```

### Architecture Notes

**Microservices pattern**: Each service in `rs/services/` is an independent Axum HTTP server that owns its own database schema. The gateway service proxies all external traffic, routing by path prefix (e.g., `/auth/people` -> `http://auth:8000/people`). The docs service auto-discovers services and aggregates their OpenAPI specs into a unified Swagger UI.

**Schema-per-service isolation**: Each service owns and manages its own database schema. Cross-service data references use UUIDs without foreign key constraints -- referential integrity is maintained via internal API calls between services, not database-level FKs.

**Internal APIs**: Services support feature-gated internal endpoints via `service.json` variants. For example, the auth service has an `int-auth` variant that exposes `/internal/entities/*` routes when built with `--features internal-api`. The Tiltfile auto-discovers variants and can deploy them as separate containers. Internal APIs are used for cross-service data lookups.

**Service discovery**: The `service-discovery` crate reads a manifest of available services and constructs URLs based on the deployment environment (Docker Compose networking vs Railway internal networking).

**TypeScript workspace**: The `ts/` directory mirrors the `rs/` microservices pattern as a PNPM workspace. Each Next.js app lives in `ts/services/{app}/` with its own `service.json` for Tilt auto-discovery. Shared packages (UI theme, TypeScript configs) live in `ts/packages/` and are consumed as workspace dependencies. The Tiltfile discovers TypeScript services alongside Rust services using the same `service.json` convention.

---

## 3. Technology Stack

### Backend (Existing)

| Dependency | Purpose | Notes |
|---|---|---|
| `axum` 0.8 | HTTP framework | Per-service routers with utoipa OpenAPI |
| `tokio` 1.48 (full features) | Async runtime | |
| `sqlx` 0.8 (postgres, migrate, uuid, chrono) | Database | Compile-time checked queries |
| `reqwest` 0.12 | HTTP client | Used by gateway proxy + service-to-service |
| `serde` / `serde_json` 1.0 | Serialization | |
| `tower-http` 0.6 (cors, trace, timeout) | Middleware | |
| `tracing` 0.1 / `tracing-subscriber` 0.3 | Logging | |
| `uuid` 1.0 (v4, serde) | IDs | |
| `chrono` 0.4 | Timestamps | |
| `thiserror` 2.0 | Error types | Per-crate error enums |
| `anyhow` 1.0 | Error propagation | In handlers/bin only |
| `dotenvy` 0.15 | Env loading | Dev only |
| `async-trait` 0.1 | Async trait support | For repository traits |
| `utoipa` 5 + `utoipa-axum` 0.1 | OpenAPI docs | Auto-generated from annotations |
| `redis` 1.0.1 | Cache | ConnectionManager abstraction |
| `alloy` 1.x | Ethereum interaction | Provider, contract calls, ABI encoding (via evm crate) |
| `turnkey_client` 0.6 | Turnkey API SDK | Typed activity methods, retry logic, P-256 auth |
| `turnkey_api_key_stamper` 0.6 | Turnkey request signing | P-256 ECDSA stamps for API authentication |
| `subtle` 2.6 | Constant-time comparison | Timing-safe credential verification in auth service |

### Backend (To Add)

| Dependency | Purpose | Notes |
|---|---|---|
| `axum-test` | Integration tests | |

Note: `jsonwebtoken` is already in workspace dependencies (used by auth service).

### Frontend (PNPM Workspace: `ts/`)

| Dependency | Purpose | Location |
|---|---|---|
| `pnpm` | Package manager (workspace-native) | Root |
| `next` latest stable | Framework | `ts/services/console` |
| `@mantine/core` + `@mantine/hooks` + `@mantine/form` + `@mantine/notifications` | UI components | `ts/packages/ui` |
| `iron-session` | Encrypted cookie sessions (deployer console auth) | `ts/services/console` |
| `swr` | Data fetching / cache invalidation | `ts/services/console` |
| `zod` | Runtime validation (API request/response schemas) | `ts/services/console` |
| `@types/cookie` | Type definitions for iron-session's cookie dependency | `ts/services/console` (devDependency) |

**To add when implementing end-user auth (Phase 3):**

| Dependency | Purpose | Location |
|---|---|---|
| `@turnkey/sdk-browser` | End-user Turnkey interactions | `ts/services/console` |
| `@turnkey/sdk-react` | Turnkey provider/hooks | `ts/services/console` |
| `typescript` | | Root |
| `biome` | Linting + formatting (replaces ESLint + Prettier) | Root |

### Infrastructure (Existing)

| Tool | Purpose |
|---|---|
| PostgreSQL 17 + pgvector | Primary database (with health checks, persistent volume) |
| Redis 8 | Cache (password-protected, health checks, persistent volume) |
| MinIO | S3-compatible object storage for file uploads (API port 9000, console 9001) |
| Docker + Docker Compose | Local infrastructure |
| Tilt | Local dev orchestration with auto-discovery and hot reload |

### Infrastructure (To Add for Production)

| Tool | Purpose |
|---|---|
| Railway | Production hosting (backend services, frontend, managed postgres) |
| Railway Storage Buckets | Production file storage (replaces MinIO) |

---

## 4. Domain Model

### Service Ownership Map

| Domain Concept | Owning Service | Schema |
|---|---|---|
| Person, Org, OrgMember | auth | `auth` |
| EntityWallet, EntityAppWallet, DelegatedAccount | wallets | `wallets` |
| AppRegistration, Contract, RoleGrant | registry | `registry` |
| ApiRequestLog, ContractEvent | analytics | `analytics` |
| File, Doc, Image, Media | assets | `assets` |

Cross-service references (e.g., `org_id` in `registry.app_registrations`) are stored as plain UUIDs without FK constraints. Referential integrity is enforced via internal API calls.

Auth codes are stored in Redis with TTL (not PostgreSQL). See Section 5 for key format details.

### Person (auth service -- existing as `auth.people`)

A person who has signed up for the platform. Stored with `auth_provider_name` and `auth_provider_ref` for authentication provider abstraction. Inherits from `auth.entities` (which provides `id`, `display_name`, `description`, `created_at`, `updated_at`). Has a personal workspace (org with `visibility = 'personal'`) created automatically via database trigger on signup.

For EVMAuth integration, each person will also have a Turnkey sub-org (managed by the wallets service) that is permanently theirs regardless of org membership. Can be a member of many orgs, and simultaneously an end user of apps deployed by those orgs or others.

### Organization (auth service -- existing as `auth.orgs`)

The identity entity that represents a group. Has `owner_id` (FK to `auth.people`), `visibility` ('personal', 'private', 'public'), and inherits from `auth.entities`. The personal workspace (`visibility = 'personal'`) created on signup maps to the "default personal org" concept. Database triggers enforce one-owner-per-org and prevent deletion of the last personal workspace.

Members have roles: `owner`, `admin`, `member`.

- `owner`: can transfer org ownership, delete org, manage members
- `admin`: can deploy contracts, manage app registrations, grant/revoke platform operator access
- `member`: read-only access to console

### OrgMember (auth service -- existing as `auth.orgs_people`)

Junction table with `org_id`, `member_id`, `role`. Database triggers sync the owner role with `auth.orgs.owner_id` and validate membership constraints.

### EntityWallet (wallets service -- to build)

A Turnkey sub-org and HD wallet for any entity (person or org). One per entity. Contains the Turnkey sub-org ID, HD wallet ID, first-derived wallet address (index 0, used as platform identity), and optional delegated account user ID. For orgs, the delegated account holds a P256 API key scoped by Turnkey policy to `ACTIVITY_TYPE_SIGN_RAW_PAYLOAD` only. Replaces both `person_turnkey_refs` and `org_wallets`.

### EntityAppWallet (wallets service -- to build)

An HD wallet account derived for a specific (entity, app_registration) pair. For orgs, this derived address becomes the EVMAuth default admin for the app's proxy contract. For people, this is the end user's identity within that app. Created when an app registration is created (for orgs) or when an end user first authenticates (for people). Replaces `person_app_wallets` and adds org app wallet support.

### AppRegistration (registry service -- partial)

An OAuth-like client registration under an org. Represents one application that will use EVMAuth for authorization. Contains the client ID, allowed callback URLs, a reference to the EVMAuth contract, and the set of token IDs relevant to this app.

There are no client secrets. Access to the authorization query API is controlled by ERC-6909 token holdings on the platform's own EVMAuth contract. The `client_id` is a public lookup key only.

One org can have many app registrations, each pointing to a different EVMAuth proxy contract.

When an app registration is created, the wallets service derives an org app-admin wallet account for that app from the org's HD wallet. This derived address becomes the EVMAuth default admin for the app's deployed proxy contract.

### Contract (registry service -- partial)

An EVMAuth beacon proxy deployed on Radius. Belongs to an org. The proxy is deployed by the platform operator wallet (which pays gas and owns the proxy at the EVM level for upgradeability), but the EVMAuth default admin role is set to the org's app-specific derived address (from EntityAppWallet). The platform can upgrade the beacon implementation (affecting all proxies), but only the org controls EVMAuth role administration on their contract. Contains the on-chain contract address, deployment transaction hash, and the beacon implementation address at time of deployment.

### PlatformContract

The platform's own EVMAuth proxy contract (a BeaconProxy), deployed on Radius. The `initialize()` function sets roles at deployment time via the `roleGrants` array parameter. Token IDs on this contract represent platform capabilities:

| Token ID | Capability |
|---|---|
| 1 | API access (`GET /accounts` endpoint) |
| 2 | Contract deployment |
| 3 | Org admin actions |

**Role assignments on the platform proxy:**

| EVMAuth Role | Held by | Why |
|---|---|---|
| `DEFAULT_ADMIN_ROLE` | Beacon owner wallet | Can grant/revoke all other roles; time-delayed transfer via `beginDefaultAdminTransfer`. Highest privilege -- rarely used. |
| `TOKEN_MANAGER_ROLE` | Beacon owner wallet | Can call `createToken`, `updateToken`, `setTokenMetadata`. Defines new capability token types (rare). |
| `ACCESS_MANAGER_ROLE` | Beacon owner wallet | Can call `freezeAccount`/`unfreezeAccount`. Emergency action (rare). |
| `TREASURER_ROLE` | Beacon owner wallet | Can call `setTreasury`. Changes where purchase revenue goes (rare). |
| `MINTER_ROLE` | Platform operator wallet | Can call `mint(to, id, amount)`. Mints capability tokens on org creation (frequent). |
| `BURNER_ROLE` | Platform operator wallet | Can call `burn(from, id, amount)`. Burns capability tokens on org deletion/revocation (frequent). |

When a deployer registers an org, the platform operator mints capability tokens to the org's first HD wallet account (index 0, derived at org creation time). This address serves as the org's platform identity. Revoking access is a burn -- no secrets to rotate, no database records to invalidate.

### File / Doc / Image / Media (assets service -- existing as `assets.*`)

The assets service manages uploaded files with S3/MinIO backend. The `assets.files` base table supports org-scoped and user-scoped uploads. Subtypes (`docs`, `images` with dimensions, `media` with duration) inherit from `files`. Used for features like company logo uploads.

---

## 5. Database Schema

The database uses schema-per-service isolation. Each microservice owns its schema and is the only service that reads from or writes to it. All tables use UUID primary keys via `gen_random_uuid()`. All timestamps are `timestamptz`. The `moddatetime` trigger handles automatic `updated_at` updates.

All migrations live in `rs/services/db/migrations/` and are run by the `db` service. Migration file names are prefixed with the owning schema (e.g., `20260310000001_wallets_entity_wallets.sql`).

### `auth` Schema (auth service -- existing)

The following tables are already implemented with full migration support, triggers, and indexes:

- `auth.entities` -- Base table: `id`, `display_name`, `description`, `created_at`, `updated_at`
- `auth.people` -- Inherits from entities. Adds: `auth_provider_name`, `auth_provider_ref`, `primary_email`. Unique on `(primary_email, auth_provider_name)`. Trigger auto-creates personal workspace on insert.
- `auth.orgs` -- Inherits from entities. Adds: `owner_id` (FK people), `visibility` ('personal'/'private'/'public'). Unique index ensures one personal org per owner. Triggers sync owner role in `orgs_people` and prevent deletion of last personal workspace.
- `auth.orgs_people` -- Junction: `org_id`, `member_id`, `role`, `created_at`, `updated_at`. PK `(org_id, member_id)`. Unique index enforces one owner per org. Triggers validate membership constraints.

**Redis-stored data (auth service):**

Auth codes for the PKCE token exchange flow are stored in Redis, not PostgreSQL. Redis TTL handles expiration automatically -- no background cleanup task is needed.

```
Key:    auth_code:{sha256_hex}
Value:  JSON { app_registration_id, entity_app_wallet_id, code_challenge, redirect_uri, state }
TTL:    Configurable via AUTH_CODE_TTL_SECS (default 30 seconds)
```

- Create: `SET auth_code:{hash} {json} EX {ttl}`
- Exchange: `GET auth_code:{hash}` then `DEL auth_code:{hash}` (atomic via Redis transaction or Lua script)
- No cleanup task needed -- expired keys vanish automatically

### `wallets` Schema (wallets service -- to build)

```sql
-- Migration: wallets_create_schema
CREATE SCHEMA IF NOT EXISTS wallets;

-- Migration: wallets_entity_wallets

-- Entity wallet: Turnkey sub-org and HD wallet for any entity (person or org).
-- One per entity. Account index 0 is the entity's platform identity address.
-- For orgs, the delegated account holds a P256 API key scoped by Turnkey policy
-- to ACTIVITY_TYPE_SIGN_RAW_PAYLOAD only.
CREATE TABLE wallets.entity_wallets (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id                   UUID NOT NULL UNIQUE,   -- references auth.entities (no FK)
    turnkey_sub_org_id          TEXT NOT NULL UNIQUE,
    turnkey_wallet_id           TEXT NOT NULL,           -- Turnkey HD wallet ID
    wallet_address              TEXT NOT NULL,           -- first derived account (index 0), platform identity
    turnkey_delegated_user_id   TEXT,                    -- nullable: orgs have this, people typically don't
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_entity_wallets_pagination ON wallets.entity_wallets(created_at, id);

CREATE TRIGGER but_entity_wallets_moddatetime
    BEFORE UPDATE ON wallets.entity_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);

-- Migration: wallets_entity_app_wallets

-- Per-app derived HD wallet accounts for both people and orgs.
-- For orgs, the derived address becomes the EVMAuth default admin for the
-- app's proxy contract. For people, the derived address is the end user's
-- identity within that app.
CREATE TABLE wallets.entity_app_wallets (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id               UUID NOT NULL,              -- references auth.entities (no FK)
    app_registration_id     UUID NOT NULL,              -- references registry.app_registrations (no FK)
    wallet_address          TEXT NOT NULL,               -- EIP-55 checksummed
    turnkey_account_id      TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (entity_id, app_registration_id),
    UNIQUE (wallet_address)
);
CREATE INDEX idx_entity_app_wallets_entity_id ON wallets.entity_app_wallets(entity_id);
CREATE INDEX idx_entity_app_wallets_address ON wallets.entity_app_wallets(wallet_address);
CREATE INDEX idx_entity_app_wallets_pagination ON wallets.entity_app_wallets(created_at, id);

CREATE TRIGGER but_entity_app_wallets_moddatetime
    BEFORE UPDATE ON wallets.entity_app_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
```

### `registry` Schema (registry service -- to build)

```sql
-- Migration: registry_create_schema
CREATE SCHEMA IF NOT EXISTS registry;

-- Migration: registry_app_registrations

-- App registrations: one per application using EVMAuth.
-- No client secret -- access is controlled by ERC-6909 token holdings
-- on the platform's own EVMAuth contract.
CREATE TABLE registry.app_registrations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id              UUID NOT NULL,              -- references auth.orgs (no FK)
    name                TEXT NOT NULL,
    client_id           TEXT NOT NULL UNIQUE,        -- public lookup key (random, URL-safe)
    callback_urls       TEXT[] NOT NULL DEFAULT '{}',
    relevant_token_ids  BIGINT[] NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_app_registrations_org_id ON registry.app_registrations(org_id);
CREATE INDEX idx_app_registrations_pagination ON registry.app_registrations(created_at, id);

CREATE TRIGGER but_app_registrations_moddatetime
    BEFORE UPDATE ON registry.app_registrations
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);

-- Migration: registry_contracts

-- Deployed EVMAuth proxy contracts on Radius.
CREATE TABLE registry.contracts (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  UUID NOT NULL,          -- references auth.orgs (no FK)
    app_registration_id     UUID,                   -- references registry.app_registrations (nullable)
    name                    TEXT NOT NULL,
    address                 TEXT NOT NULL UNIQUE,    -- on-chain address (EIP-55 checksummed)
    chain_id                TEXT NOT NULL,
    beacon_address          TEXT NOT NULL,
    deploy_tx_hash          TEXT NOT NULL,
    deployed_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_contracts_org_id ON registry.contracts(org_id);
CREATE INDEX idx_contracts_pagination ON registry.contracts(created_at, id);

CREATE TRIGGER but_contracts_moddatetime
    BEFORE UPDATE ON registry.contracts
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);

-- EVMAuth role grants: platform operator roles granted on deployer contracts.
-- Each row tracks a grantRole/revokeRole lifecycle for a specific role.
CREATE TABLE registry.role_grants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id     UUID NOT NULL REFERENCES registry.contracts(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,           -- EVMAuth role name (e.g. 'MINTER_ROLE')
    grant_tx_hash   TEXT NOT NULL,
    revoke_tx_hash  TEXT,
    active          BOOLEAN NOT NULL DEFAULT true,
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_role_grants_contract_id ON registry.role_grants(contract_id);

CREATE TRIGGER but_role_grants_moddatetime
    BEFORE UPDATE ON registry.role_grants
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
```

### `analytics` Schema (analytics service -- to build)

```sql
-- Migration: analytics_create_schema
CREATE SCHEMA IF NOT EXISTS analytics;

-- Migration: analytics_api_requests

-- Log of authorization query API requests. Used for console analytics.
CREATE TABLE analytics.api_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id       TEXT NOT NULL,              -- app registration client_id
    signer          TEXT NOT NULL,              -- address that signed the request
    principal       TEXT NOT NULL,              -- address being queried
    contract        TEXT NOT NULL,              -- contract being queried
    delegate        TEXT,                       -- delegate address (if provided)
    response_code   SMALLINT NOT NULL,          -- HTTP status code
    latency_ms      INTEGER NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_api_requests_client_id ON analytics.api_requests(client_id);
CREATE INDEX idx_api_requests_created_at ON analytics.api_requests(created_at);
CREATE INDEX idx_api_requests_pagination ON analytics.api_requests(created_at, id);

-- Migration: analytics_contract_events

-- Indexed on-chain events from deployed EVMAuth contracts.
CREATE TABLE analytics.contract_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id     UUID NOT NULL,              -- references registry.contracts (no FK)
    event_type      TEXT NOT NULL,              -- e.g., 'Transfer', 'OperatorSet', 'Approval'
    tx_hash         TEXT NOT NULL,
    block_number    BIGINT NOT NULL,
    log_index       INTEGER NOT NULL,
    event_data      JSONB NOT NULL,             -- decoded event parameters
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_contract_events_contract_id ON analytics.contract_events(contract_id);
CREATE INDEX idx_contract_events_type ON analytics.contract_events(event_type);
CREATE INDEX idx_contract_events_block ON analytics.contract_events(block_number);
CREATE INDEX idx_contract_events_pagination ON analytics.contract_events(created_at, id);
```

### `assets` Schema (assets service -- existing)

- `assets.files` -- Base: `id`, `org_id`, `uploader_id`, `object_key` (unique S3 path), `file_name`, `mime_type`, `size_bytes`, `created_at`, `updated_at`
- `assets.docs` -- Inherits from files
- `assets.images` -- Inherits from files. Adds: `height`, `width`
- `assets.media` -- Inherits from images. Adds: `duration_ms`

### Naming Conventions (Established)

| Item | Convention | Example |
|---|---|---|
| Schemas | service name | `auth`, `wallets`, `registry`, `analytics`, `assets` |
| Tables | plural snake_case | `auth.people`, `registry.contracts` |
| Join tables | `{table1}_{table2}` | `auth.orgs_people` |
| Indexes | `idx_{table}_{columns}` | `idx_people_pagination` |
| Unique constraints | `uq_{table}_{columns}` | `uq_people_email_provider` |
| Triggers | `{timing}_{table}_{action}` | `but_people_moddatetime` |
| Trigger functions | `{schema}.tfn_{table}_{action}` | `auth.tfn_people_create_personal_workspace` |
| Migration files | `{timestamp}_{schema}_{description}.sql` | `20260310000001_wallets_entity_wallets.sql` |

---

## 6. Backend -- Rust / Axum Microservices

### Existing Architecture

The backend follows a microservices pattern. Each service is an independent Axum HTTP server with its own:
- `AppState` (db pool, redis, service-specific clients)
- Route definitions with utoipa OpenAPI annotations
- Domain types, DTOs (request/response), repository layer
- Error handling (`ApiError` -> HTTP response mapping)
- Health check endpoint
- Internal API variant via feature flags

Services are accessed through the **gateway**, which routes by path prefix. The **docs** service aggregates OpenAPI specs from all services into a unified Swagger UI.

### Service Architecture

```
                    ┌──────────────┐
      External      │   Gateway    │     Port 8000 (external)
      Traffic  ───> │   (proxy)    │
                    └──────┬───────┘
                           │
         ┌─────────┬───────┼─────────┬───────────┐
         │         │       │         │           │
   ┌─────┴───┐ ┌───┴───┐ ┌┴──────┐ ┌┴────────┐ ┌┴────┐
   │  Auth   │ │Wallets│ │Regist.│ │Analytics│ │Asset│
   │  :8000  │ │ :8000 │ │ :8000 │ │  :8000  │ │:8000│
   └────┬────┘ └───┬───┘ └───┬───┘ └────┬────┘ └──┬──┘
        │          │         │          │         │
   ┌────┴──────────┴─────────┴──────────┴─────────┘
   │              PostgreSQL + Redis              │
   │   auth.*   wallets.*  registry.*  analytics.*│
   └──────────────────────────────────────────────┘
```

The gateway resolves service URLs based on environment:
- Docker Compose: `http://<service>:8000`
- Railway: `http://<service>.railway.internal:8000`

### Cross-Service Communication

Services call each other via internal HTTP APIs (feature-gated `/internal/*` routes). The gateway is not used for internal calls -- services resolve each other directly via service discovery.

| Caller | Callee | Purpose |
|---|---|---|
| auth | wallets (internal) | Create entity wallet (Turnkey sub-org + HD wallet) on person signup |
| auth | wallets (internal) | Create entity wallet for org on org creation |
| auth | wallets (internal) | Create entity app wallet for person during PKCE flow |
| auth | wallets (internal) | Add user to org's Turnkey sub-org when invited to join org |
| auth | registry (internal) | Validate `client_id` and `redirect_uri` during PKCE flow |
| registry | wallets (internal) | Create entity app wallet when app registration is created |
| registry | wallets (internal) | Look up org's entity app wallet address for contract deployment |
| registry | wallets (internal) | Sign transactions via delegated account |
| registry | analytics (internal) | Log API request after accounts query |
| analytics | registry (internal) | Look up contract metadata for event indexing |

### Auth Service (Existing + To Extend)

**Schema owned:** `auth`

**Currently implemented:**
- People CRUD: `GET/POST /people`, `GET/PUT/DELETE /people/{id}`
- Orgs CRUD: `GET/POST /orgs`, `GET/PUT/DELETE /orgs/{id}`
- Org members: `GET/POST /orgs/{id}/members`, `PUT/DELETE /orgs/{org_id}/members/{member_id}`
- Internal entities API (feature-gated): `GET /internal/entities`, `GET/DELETE /internal/entities/{id}`
- Health check, OpenAPI spec
- Repository pattern, filters, pagination

**To add:**

```
/auth/                              # Via gateway: /auth/*
├── auth/                           # Authentication flows
│   ├── POST   /signup              # Deployer signup
│   ├── POST   /login               # Deployer login
│   ├── POST   /callback            # OIDC callback
│   ├── POST   /logout              # Clear session
│   ├── GET    /end-user/authorize  # Initiate end-user PKCE auth
│   └── POST   /end-user/token      # Exchange auth code + PKCE verifier -> JWT
│
├── .well-known/
│   └── GET    /jwks.json           # RS256 public key for JWT verification
│
├── people/
│   ├── GET    /me                  # Current person profile
│   └── PATCH  /me                  # Update display name
│
└── internal/                       # Feature-gated
    ├── GET    /people/{id}         # Lookup person by ID (for other services)
    └── GET    /orgs/{id}           # Lookup org by ID (for other services)
```

**AppState (extended):**

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub jwt_keys: Option<Arc<JwtKeys>>,   // RS256 keypair (optional in dev)
    pub http_client: reqwest::Client,     // Internal service calls
    pub config: Arc<Config>,
    pub evm: Arc<evm::EvmClient>,         // Read-only EVM client for platform contract
}
```

**Config includes:** `AUTH_CODE_TTL_SECS` (default 30) -- configurable TTL for PKCE authorization codes stored in Redis.

### Wallets Service (To Build)

**Schema owned:** `wallets`

Manages all Turnkey sub-org lifecycle, wallet creation, and signing operations. The only service that talks to the Turnkey API.

```
/wallets/                           # Via gateway: /wallets/*
├── GET    /health
├── GET    /openapi.json
│
├── orgs/{org_id}/wallet/           # Org wallet management
│   └── GET    /                    # Get org wallet info (address, status)
│
├── me/wallets/                     # End-user wallet self-service
│   ├── GET    /                    # List all app wallets for current person
│   └── GET    /{app_id}            # Get wallet details for specific app
│
└── internal/                       # Feature-gated
    ├── POST   /entity-wallet           # Create Turnkey sub-org + HD wallet for entity
    ├── GET    /entity-wallet/{entity_id} # Look up entity wallet
    ├── POST   /entity-app-wallet       # Derive HD wallet account for (entity, app)
    ├── GET    /entity-app-wallet/{entity_id}/{app_id} # Look up entity app wallet
    ├── POST   /org-sub-org-user        # Add a user to an org's Turnkey sub-org
    ├── POST   /sign                    # Sign payload via delegated account
    └── POST   /send-tx                 # Sign + broadcast transaction via delegated account
```

**AppState:**

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub turnkey: Arc<turnkey_client::TurnkeyClient<TurnkeyP256ApiKey>>,
    pub turnkey_parent_org_id: String,
    pub evm: Arc<EvmClient>,
    pub config: Arc<Config>,
}
```

The wallets service uses the official Turnkey Rust SDK (`turnkey_client` + `turnkey_api_key_stamper`) directly -- no wrapper crate. The SDK provides typed activity methods (e.g., `create_sub_organization`, `create_wallet`, `sign_raw_payload`), built-in retry logic with exponential backoff, and P-256 ECDSA request signing via the `Stamp` trait.

### Registry Service (To Build)

**Schema owned:** `registry`

Manages app registrations, deployed contracts, role grants, and the authorization query endpoint.

```
/registry/                          # Via gateway: /registry/*
├── GET    /health
├── GET    /openapi.json
│
├── orgs/{org_id}/apps/             # App registrations
│   ├── POST   /
│   ├── GET    /
│   ├── GET    /{app_id}
│   ├── PATCH  /{app_id}
│   └── DELETE /{app_id}
│
├── orgs/{org_id}/contracts/        # Contract management
│   ├── POST   /                    # Deploy new EVMAuth proxy
│   ├── GET    /
│   ├── GET    /{contract_id}
│   ├── POST   /{contract_id}/roles          # Grant a role
│   ├── GET    /{contract_id}/roles          # List role grants
│   └── DELETE /{contract_id}/roles/{role_grant_id}  # Revoke a role
│
├── accounts/                       # Authorization query (ERC-712 authenticated)
│   └── GET    /{address}
│
└── internal/                       # Feature-gated
    ├── GET    /apps/by-client-id/{client_id}  # Lookup by client_id
    └── GET    /contracts/{id}                 # Lookup contract by ID
```

**AppState:**

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub evm: Arc<EvmClient>,
    pub http_client: reqwest::Client,
    pub config: Arc<Config>,
}
```

### Analytics Service (To Build)

**Schema owned:** `analytics`

Indexes contract events from the chain and logs API requests. Exposes metrics for the developer console.

```
/analytics/                         # Via gateway: /analytics/*
├── GET    /health
├── GET    /openapi.json
│
├── orgs/{org_id}/usage/            # Console analytics
│   ├── GET    /requests            # API request volume/breakdown
│   └── GET    /events              # Contract event history
│
└── internal/                       # Feature-gated
    └── POST   /requests            # Log an API request (called by registry)
```

**AppState:**

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub evm: Arc<EvmClient>,        // For event indexing
    pub config: Arc<Config>,
}
```

### Auth Middleware

Three middleware layers used across services:

1. **`RequireSession`** (auth service) -- validates the platform session JWT (set as an HTTP-only cookie). Extracts `person_id` into request extensions.

2. **`RequireOrgRole(role)`** (auth service, registry service) -- confirms the current person is a member of the org in the route path with at least the required role. Calls auth internal API if needed.

3. **`RequireErc712Auth`** (registry service) -- authenticates requests to the `/accounts` endpoint. The caller must include three headers:
    - `X-Client-Id` -- identifies the app registration (public lookup key)
    - `X-Signer` -- the address that produced the signature
    - `X-Signature` -- ERC-712 signature over a canonical request digest

   The middleware recovers the signer from the signature, then calls `balanceOf(platform_contract, signer, TOKEN_ID_API_ACCESS)` on Radius. If the balance is zero, the request is rejected.

### JWT Strategy

**Platform session JWT** (deployer console): RS256, signed with platform private key. Claims:

```json
{
  "iss": "https://api.evmauth.io",
  "sub": "<person_platform_id>",
  "type": "session",
  "exp": "<8 hours>",
  "iat": "<now>"
}
```

**End-user app JWT** (issued to deployer apps): RS256, same signing key. Claims:

```json
{
  "iss": "https://auth.evmauth.io",
  "sub": "<person_platform_id>",
  "aud": "<client_id>",
  "type": "end_user",
  "wallet": "0x...",
  "contract": "0x...",
  "chain_id": "<radius_chain_id>",
  "exp": "<configurable by deployer, default 1 hour>",
  "iat": "<now>"
}
```

The public key for JWT verification is exposed at `GET /auth/.well-known/jwks.json` (via gateway).

### Turnkey SDK Integration -- Existing

The wallets service uses the official Turnkey Rust SDK directly (`turnkey_client` 0.6 + `turnkey_api_key_stamper` 0.6). There is no custom turnkey wrapper crate -- the SDK provides everything needed:

- **`turnkey_client::TurnkeyClient<S: Stamp>`** -- typed activity methods with built-in retry logic (`RetryConfig` with exponential backoff, default 5 attempts)
- **`turnkey_api_key_stamper::TurnkeyP256ApiKey`** -- P-256 ECDSA request signing (replaces placeholder stamp implementation)
- **`TurnkeyClientError`** -- structured error enum with variants for HTTP failures, activity failures, stamper errors, retry exhaustion, etc.
- **Typed SDK intents and results** -- e.g., `CreateSubOrganizationIntentV7`, `CreateWalletIntent`, `SignRawPayloadIntentV2` with concrete result types (`ActivityResult<T>`)

The client is constructed in the wallets service `main.rs`:

```rust
let api_key = TurnkeyP256ApiKey::from_strings(
    &config.turnkey.api_private_key,
    Some(&config.turnkey.api_public_key),
)?;
let turnkey = turnkey_client::TurnkeyClient::builder()
    .api_key(api_key)
    .base_url(&config.turnkey.api_base_url)
    .build()?;
```

Handlers use SDK intent types directly (e.g., `CreateSubOrganizationIntentV7`, `WalletAccountParams`) and SDK enum types (e.g., `Curve::Secp256k1`, `AddressFormat::Ethereum`, `PayloadEncoding::Hexadecimal`).

### EVM Crate (`rs/crates/evm/`) -- Existing

Uses Alloy to interact with Radius. Provides a read-only HTTP provider and typed EVMAuth6909 contract bindings. Used by the auth service (for capability token minting calldata) and will be used by the registry and analytics services.

`EvmConfig` is a plain data struct -- it does not read environment variables. The consuming service is responsible for populating it from its own config source.

Signing is not handled by the evm crate. The wallets service owns Turnkey signing; other services encode calldata via `encode_mint()` and POST it to the wallets service `/internal/sign` endpoint.

```rust
pub struct EvmConfig {
    pub rpc_url: String,
    pub platform_contract_address: Address,
    pub chain_id: u64,
}

pub struct EvmClient { /* Alloy HTTP provider + config */ }

impl EvmClient {
    pub fn new(config: EvmConfig) -> Result<Self, EvmError>;
    pub async fn balance_of(&self, account: Address, token_id: U256) -> Result<U256>;  // 10s timeout
    pub async fn is_operator(&self, owner: Address, spender: Address) -> Result<bool>;  // 10s timeout
    pub fn encode_mint(to: Address, token_id: U256, amount: U256) -> Bytes;  // static
    pub fn encode_initialize(admin: Address, treasury: Address, operator: Address, uri: &str) -> Bytes;  // static
    pub fn encode_grant_role(role: FixedBytes<32>, account: Address) -> Bytes;  // static
    pub fn encode_revoke_role(role: FixedBytes<32>, account: Address) -> Bytes;  // static
}

pub mod roles {
    pub fn all_operator_roles() -> Vec<FixedBytes<32>>;  // TOKEN_MANAGER, ACCESS_MANAGER, TREASURER, MINTER, BURNER
    pub fn role_name_to_bytes(name: &str) -> Option<FixedBytes<32>>;
}

pub fn encode_beacon_proxy_deploy(beacon: Address, init_data: Bytes) -> Result<Bytes, EvmError>;
```

All RPC calls are wrapped in `tokio::time::timeout` (10s) to prevent hangs from unresponsive nodes. The `EvmError` enum includes a `Timeout` variant for this.

`encode_initialize()` builds the ABI-encoded calldata for the proxy's `initialize()` function, granting all non-admin roles to the platform operator and setting the org's app-specific address as default admin.

Future additions: `balances_for` (batch query for multiple token IDs).

### Accounts Endpoint (Authorization Query -- Registry Service)

```
GET /registry/accounts/{address}?contract={contract_address}&delegate={delegate_address}
X-Client-Id: <app_registration_client_id>
X-Signer:    <address that produced the signature>
X-Signature: <ERC-712 signature over canonical request digest>
```

#### ERC-712 Request Signing

```
Domain:
  name:    "EVMAuth API"
  version: "1"
  chainId: <radius_chain_id>

Type: AccountsQuery
  address   address
  contract  address
  clientId  string
  nonce     uint256   -- unix timestamp in seconds (reject if >30s old)
```

#### Handler Logic

1. Validate `X-Signature` is a valid ERC-712 signature recovering to `X-Signer`
2. Reject if `nonce` (timestamp) is older than 30 seconds
3. Call `balanceOf(platform_contract, X-Signer, TOKEN_ID_API_ACCESS)` on Radius -- reject with `403` if zero
4. Verify `X-Client-Id` resolves to an app registration whose org's entity app wallet address (from `wallets.entity_app_wallets`) matches `X-Signer` (or has `X-Signer` as an approved operator via `isOperator`)
5. If `delegate` query param is present: call `is_operator(contract, address, delegate)` -- reject with `403` if false
6. Query `balance_of` for each token ID in `relevant_token_ids` for the principal address
7. Call analytics internal API to log the request
8. Return response

```json
{
  "address": "0x<principal>",
  "contract": "0x<contract>",
  "chain_id": "<radius_chain_id>",
  "chain_name": "Radius",
  "delegate": "0x<delegate or null>",
  "tokens": [
    { "id": "3", "balance": "1" },
    { "id": "7", "balance": "0" }
  ],
  "queried_at": "2026-03-10T12:00:00Z"
}
```

No caching. Always reads live chain state.

---

## 7. Frontend -- TypeScript / Next.js Workspace

### Workspace Structure

The `ts/` directory is a PNPM workspace that mirrors the `rs/` microservices pattern. Each Next.js app is a standalone service under `ts/services/`, and shared code lives in `ts/packages/`.

```yaml
# ts/pnpm-workspace.yaml
packages:
  - "services/*"
  - "packages/*"
```

The Tiltfile auto-discovers TypeScript services the same way it discovers Rust services: by scanning `ts/services/` for directories containing a `service.json`. Each TypeScript service uses `pnpm dev` for hot reload in development.

### Shared Packages

| Package | Path | Purpose |
|---|---|---|
| `@evmauth/ui` | `ts/packages/ui` | Mantine theme configuration, custom components, shared styles |
| `@evmauth/tsconfig` | `ts/packages/tsconfig` | Base TypeScript configurations (extends per-service) |

Shared packages are consumed as workspace dependencies (e.g., `"@evmauth/ui": "workspace:*"` in each service's `package.json`).

### Console Service (`ts/services/console`)

This is the primary frontend application. It serves the developer console and the hosted end-user auth UI.

#### Architecture Principles

- The frontend is a **dumb interface**. It renders data, collects input, and calls the backend via the Next.js proxy. It contains no business logic.
- All API calls from frontend components go to `/api/proxy/[...path]` which is a Next.js route handler that forwards the request to the Rust backend **gateway**, attaching the session cookie. This keeps the backend URL out of the browser entirely.
- Mantine handles all UI components, theming, and responsive layout via the `@evmauth/ui` package. No custom CSS unless absolutely necessary.
- Biome enforces consistent formatting and linting. The root `ts/biome.json` is shared across all services and packages.

#### Next.js API Proxy

```typescript
// src/app/api/proxy/[...path]/route.ts
// Reads iron-session server-side via getIronSession.
// Rejects unauthenticated requests with 401.
// Forwards requests to BACKEND_URL (gateway) with X-Person-Id header from session.
// Strips raw browser cookies -- never forwards Cookie header to the backend.
// Handles GET, POST, PATCH, PUT, DELETE.
// Forwards non-2xx responses with original status codes.
```

The proxy also handles the one case where the frontend does need to talk to Turnkey directly: the end-user auth callback page uses `@turnkey/sdk-browser` to decrypt the credential Turnkey returns after OIDC. This happens in the browser, then the decrypted credential is sent to the backend to complete authentication.

#### Route Overview

| Route | Description |
|---|---|
| `/` | Landing page |
| `/dashboard` | Org overview -- lists orgs, default redirects to first org |
| `/dashboard/[orgSlug]` | Org home -- members, recent activity |
| `/dashboard/[orgSlug]/apps` | List app registrations |
| `/dashboard/[orgSlug]/apps/new` | Create app registration |
| `/dashboard/[orgSlug]/apps/[appId]` | App details -- client ID, callback URLs, relevant token IDs |
| `/dashboard/[orgSlug]/contracts` | List contracts |
| `/dashboard/[orgSlug]/contracts/new` | Deploy a new contract (wizard) |
| `/dashboard/[orgSlug]/contracts/[contractId]` | Contract details -- address, role grant status, block explorer link |
| `/dashboard/[orgSlug]/members` | Manage org members |
| `/dashboard/[orgSlug]/settings` | Org settings |
| `/auth/login` | Deployer login (passkey / OAuth via Turnkey) |
| `/auth/callback` | OAuth callback (deployer) |
| `/auth/end-user/login` | Hosted end-user auth page (shown to end users of deployer apps) |
| `/auth/end-user/callback` | End-user OAuth callback; completes PKCE code issuance |
| `/auth/wallet` | End-user self-service -- key export, linked apps |

#### Session Management

The frontend uses `iron-session` for encrypted cookie-based sessions. The iron-session cookie (`evmauth-console`) stores `{ personId, email, displayName }` with `httpOnly: true`, `secure` in production, `sameSite: 'strict'`, and 8-hour max age. The `SESSION_SECRET` is loaded via `lib/config.ts` which throws if the env var is missing (no hardcoded fallback).

Next.js API routes (`/api/auth/login`, `/api/auth/signup`, `/api/auth/logout`) call the backend, validate responses with Zod schemas, then create/destroy the iron-session. Backend `Set-Cookie` headers are never forwarded to the browser -- the console manages its own session exclusively. The `/api/auth/me` route returns the iron-session data for client-side session checks.

The API proxy (`/api/proxy/[...path]`) reads the iron-session server-side, rejects unauthenticated requests, and forwards requests to the backend with an `X-Person-Id` header. Raw browser cookies are never forwarded.

`middleware.ts` protects `/dashboard/*` routes: redirects to `/auth/login` if no valid iron-session. Also redirects authenticated users away from `/auth/login` to `/dashboard`.

#### Data Fetching

Use `swr` for all console data. SWR hooks are defined in `lib/hooks.ts` (e.g., `useMe`, `useOrgs`) -- never inline `useSWR` in components. Every component consuming a SWR hook handles loading, error, and success states. Use `mutate` after write operations to revalidate all affected cache keys.

```typescript
// src/lib/api-client.ts
export const api: ApiClient = {
  get: <T>(path: string): Promise<T> => request<T>(path),
  post: <T>(path: string, body: unknown): Promise<T> => request<T>(path, { method: 'POST', body }),
  patch: <T>(path: string, body: unknown): Promise<T> => request<T>(path, { method: 'PATCH', body }),
  delete: <T>(path: string): Promise<T> => request<T>(path, { method: 'DELETE' }),
};

// src/lib/api-client.ts -- authenticate() encapsulates login-then-signup logic
export async function authenticate(email: string): Promise<void>;
```

### Adding New Frontend Services

To add a new Next.js app (e.g., an internal admin tool):

1. Create `ts/services/{name}/` with `package.json`, `service.json`, `next.config.ts`
2. Add `@evmauth/ui` and `@evmauth/tsconfig` as workspace dependencies
3. The Tiltfile auto-discovers it -- no Tiltfile changes needed (uses unified `ts/Dockerfile` with `SERVICE` build arg)
4. Run `pnpm install` from the `ts/` root to link workspace dependencies
5. No per-service Dockerfile needed -- the unified `ts/Dockerfile` handles all services

---

## 8. Authentication & Identity Architecture

### Turnkey Org Hierarchy

```
EVMAuth Platform (Turnkey parent org)
|-- Beacon Owner HD Wallet
|   +-- Account index 0: beacon owner address
|       Owns the UpgradeableBeacon. Holds DEFAULT_ADMIN_ROLE,
|       TOKEN_MANAGER_ROLE, ACCESS_MANAGER_ROLE, and TREASURER_ROLE
|       on the platform proxy. Rarely used, high-privilege.
|
|-- Platform Operator HD Wallet
|   +-- Account index 0: platform operator address
|       Deploys BeaconProxy contracts. Holds MINTER_ROLE and
|       BURNER_ROLE on the platform proxy. Frequently used by
|       the wallets service for routine operations.
|
|-- [Person sub-org] -- one per person
|   +-- Root user: passkey (primary) + OAuth (optional backup)
|   +-- HD Wallet (one per person)
|       |-- Account index 0: person's platform identity
|       |-- Account for App A (derived per app_registration_id)
|       |-- Account for App B
|       +-- ...
|
+-- [Org sub-org] -- one per organization
    |-- Root user: the org owner (passkey/OAuth credential)
    |-- Delegated Account user: platform-controlled
    |   +-- API key (P256): scoped to ACTIVITY_TYPE_SIGN_RAW_PAYLOAD only
    +-- HD Wallet (one per org)
        |-- Account index 0: org platform identity (holds capability tokens)
        |-- Account for App A (EVMAuth default admin for App A's proxy)
        |-- Account for App B (EVMAuth default admin for App B's proxy)
        +-- ...
```

### Wallet Account Types

| Account type | Lives in | Derives from | Purpose |
|---|---|---|---|
| Beacon owner account | Beacon Owner HD Wallet (parent org) | Index 0 (fixed) | Owns UpgradeableBeacon, holds high-privilege roles on platform proxy |
| Platform operator account | Platform Operator HD Wallet (parent org) | Index 0 (fixed) | Deploys proxies, pays gas, holds MINTER/BURNER on platform proxy |
| Entity platform account | Entity sub-org HD wallet | Index 0 (fixed) | Platform identity, holds capability tokens (orgs) |
| Entity app account | Entity sub-org HD wallet | Per app_registration_id | EVMAuth default admin (orgs) or end-user identity (people) |

### Deployer Signup / Login Flow

1. Deployer visits `/auth/login`
2. Platform presents two options: **Passkey** (primary, recommended) or **Continue with Google/Apple** (OAuth)
3. For a new user via passkey:
   a. Frontend uses `@turnkey/sdk-browser` to create a passkey credential
   b. `POST /auth/auth/signup` with attestation and email
   c. Auth service calls wallets internal API to create entity wallet (Turnkey sub-org + HD wallet) for the person
   d. Auth service inserts `auth.people` row (trigger auto-creates personal workspace org)
   e. Auth service calls wallets internal API to create entity wallet + delegated account for the personal workspace org
   f. Wallets service creates Turnkey sub-org, adds delegated account user with signing-only policy, stores in `wallets.entity_wallets`
   g. Platform mints capability token(s) to the org's platform identity address (entity wallet index 0) on the platform EVMAuth contract
4. Auth service issues a platform session JWT, sets it as an HTTP-only, Secure, SameSite=Strict cookie
5. Redirect to `/dashboard`

For returning users, the passkey prompt is all that's needed.

### End-User Auth Flow (PKCE)

**Step 1 -- Authorization request** (deployer app -> EVMAuth platform):

```
GET https://auth.evmauth.io/auth/end-user/login
  ?client_id=<app_client_id>
  &redirect_uri=<registered_callback>
  &state=<random_state>
  &code_challenge=<S256_hash>
  &code_challenge_method=S256
```

**Step 2 -- Platform validates and authenticates:**
1. Auth service calls registry internal API to validate `client_id` exists and `redirect_uri` is in `callback_urls` -- reject before initiating auth if invalid
2. User authenticates via OAuth on the EVMAuth hosted UI
3. Auth service calls wallets internal API to resolve or create end-user sub-org and wallet
4. Auth service generates authorization code (32 bytes, base64url), stores `SHA-256(code)` in Redis with configurable TTL
5. Redirects to `redirect_uri?code=<plaintext_code>&state=<state>`

**Step 3 -- Token exchange** (deployer backend -> EVMAuth platform):

```
POST /auth/auth/end-user/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=<plaintext_code>
&code_verifier=<original_verifier>
&redirect_uri=<same_redirect_uri>
&client_id=<app_client_id>
```

Backend:
1. Hash the submitted `code` and look up in Redis by `auth_code:{hash}`
2. Reject if: not found (expired or already used), `redirect_uri` mismatch, or `SHA-256(code_verifier) != code_challenge`
3. Delete the key from Redis immediately -- codes are single-use
4. Auth service calls wallets internal API to get wallet address for the JWT claims
5. Issue end-user app JWT and return it

```json
{
  "access_token": "<JWT>",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Key Export (End Users)

The `/auth/wallet` page (served by frontend, data from wallets service via gateway) allows an authenticated end user to:

- View all apps they have wallets for (via `GET /wallets/me/wallets`)
- Initiate key export via Turnkey's export flow -- entirely in-browser
- Add a passkey as a backup authenticator
- View wallet address per app

### Org Membership & Turnkey Sub-Org Access

When a person is invited to join an org, they need access to the org's Turnkey sub-org to participate in signing workflows. The auth service calls wallets `POST /internal/org-sub-org-user` to add the invited person as a user in the org's Turnkey sub-org. This is separate from the `auth.orgs_people` membership record -- it's a Turnkey-level access grant that enables the person to participate in org signing operations.

The org's delegated account (platform-controlled) is used for automated signing (e.g., `grantRole`/`revokeRole` calls on deployed contracts). Individual org members interact through their own credentials added to the org's Turnkey sub-org.

---

## 9. Contract Deployment & Management

### Beacon Proxy Pattern

The platform maintains one **UpgradeableBeacon** contract on Radius, deployed and owned by the **beacon owner wallet**. It stores the address of the current EVMAuth6909 implementation. All BeaconProxy contracts read their logic from this beacon. Upgrading the beacon (`upgradeTo(newImplementation)`) upgrades all proxies atomically -- a rare, high-impact operation restricted to the beacon owner.

The **platform operator wallet** deploys new BeaconProxy contracts (one per deployer app) and holds `MINTER_ROLE`/`BURNER_ROLE` on the platform proxy for routine operations. Compromise of the platform operator is recoverable (revoke roles, rotate wallet); compromise of the beacon owner is catastrophic (attacker controls logic for every proxy).

### Deployment Flow

Deployment happens in two phases:

**Phase 1 -- App registration creation:**

1. Deployer creates an app registration in the console (`POST /registry/orgs/{org_id}/apps`)
2. Registry service calls wallets `POST /internal/entity-app-wallet` with `{ entity_id: org_id, app_registration_id }`
3. Wallets service derives a new account from the org's HD wallet, stores in `wallets.entity_app_wallets`
4. The derived address will become the EVMAuth default admin for this app's proxy contract

**Phase 2 -- Contract deployment:**

1. Deployer navigates to "Deploy Contract" (`POST /registry/orgs/{org_id}/contracts`)
2. Registry service looks up org's app wallet via `GET /internal/entity-app-wallet/{org_id}/{app_id}`
3. Registry encodes BeaconProxy deployment with init_data setting EVMAuth default admin to the org's app-specific derived address
4. Registry calls wallets `POST /internal/send-tx` to deploy (platform operator wallet pays gas and owns the proxy at the EVM level)
5. Registry inserts `registry.contracts` row
6. Console displays the new contract with a block explorer link

### Role Management

At deployment time, the `initialize()` call grants all non-admin roles (TOKEN_MANAGER, ACCESS_MANAGER, TREASURER, MINTER, BURNER) to the platform operator wallet. The org's app-specific derived address receives `DEFAULT_ADMIN_ROLE`. This means the platform can immediately operate on the contract (mint, burn, manage tokens) without a separate grant step.

Post-deployment, deployers can manage platform operator roles via the console:

**Granting a role:**
1. Deployer clicks "Grant Role" and selects a role name (e.g., MINTER_ROLE)
2. Frontend calls `POST /registry/orgs/{org_id}/contracts/{id}/roles` with `{ "role": "MINTER_ROLE" }`
3. Registry service encodes `grantRole(role, platformOperatorAddress)` and calls wallets internal API to sign and broadcast via the org's delegated account
4. Registry records the grant in `registry.role_grants`

**Revoking a role:**
1. Deployer clicks "Revoke" on an active role grant
2. Frontend calls `DELETE /registry/orgs/{org_id}/contracts/{id}/roles/{role_grant_id}`
3. Registry service encodes `revokeRole(role, platformOperatorAddress)` and calls wallets internal API to sign and broadcast
4. Registry marks the grant as revoked in `registry.role_grants`

---

## 10. Authorization Query API

This is the runtime endpoint deployers call from their own backends. Owned by the **registry service**.

### Endpoint

```
GET /registry/accounts/{address}
X-Client-Id: <app_registration_client_id>
X-Signer:    <address that produced the signature>
X-Signature: <ERC-712 signature over canonical request digest>
```

### Query Parameters

| Parameter | Required | Description |
|---|---|---|
| `contract` | Yes | The EVMAuth proxy contract address |
| `delegate` | No | If provided, verifies that `address` is an approved operator for this principal via `isOperator(address, delegate)`. Returns the principal's token holdings. |

Note the semantics: when `delegate` is present, `{address}` in the path is the **principal** (holds the tokens), and `delegate` is the agent calling on their behalf.

### Response

```json
{
  "address": "0x<principal>",
  "contract": "0x<contract>",
  "chain_id": "<radius_chain_id>",
  "chain_name": "Radius",
  "delegate": "0x<delegate or null>",
  "tokens": [
    { "id": "3", "balance": "1" },
    { "id": "7", "balance": "0" }
  ],
  "queried_at": "2026-03-10T12:00:00Z"
}
```

Token IDs with zero balance are included when they are in `relevant_token_ids`, so callers get a complete and consistent shape.

---

## 11. Local Development -- Tilt + Docker Compose

### docker-compose.yml (Existing)

Infrastructure services are defined in the root `docker-compose.yml`. The Tiltfile dynamically generates a second compose file (`.tilt/docker-compose.yml`) for the Rust microservices with hot reload.

```yaml
services:
  postgres:
    image: ankane/pgvector:v0.8.0    # PostgreSQL 17 + pgvector
    ports: ["5432:5432"]
    volumes: [db_data]

  redis:
    image: redis:8-alpine
    ports: ["6379:6379"]
    volumes: [redis_data]

  minio:
    image: minio/minio
    ports: ["9000:9000", "9001:9001"]
    volumes: [minio_data]
```

### Tiltfile (Existing)

The Tiltfile auto-discovers services in `rs/services/`. No changes needed to the Tiltfile when adding new services -- just create the service directory with a `service.json` and the Tiltfile picks it up automatically.

### TypeScript Service Auto-Discovery

The Tiltfile extends the same auto-discovery pattern to `ts/services/`. TypeScript services use a `service.json` for Tilt metadata (same schema as Rust services) and run `pnpm dev` for hot reload instead of `cargo watch`.

Each TypeScript service gets its own Docker Compose service entry with:
- Build using the unified `ts/Dockerfile` with `target: dev` and `SERVICE` build arg (mirrors the `rs/Dockerfile` pattern)
- Volume mounts for source code and workspace packages
- Named volumes for `node_modules` and `.next` cache
- Port mappings from `service.json`
- Dependencies on infrastructure and backend services

Example `ts/services/console/service.json`:

```json
{
  "ports": ["3000:3000"],
  "depends_on": ["gateway"]
}
```

No changes needed to the Tiltfile when adding new TypeScript services -- just create the service directory with a `package.json` and `service.json` and the Tiltfile picks it up automatically.

### Contract Deployment (Local Dev)

Since all signing goes through Turnkey, Anvil default private keys cannot be used for contract deployment. The Tiltfile provides three manual tasks for this workflow:

1. **`fund-wallets`** -- Sends 100 ETH from Anvil account #0 to both `BEACON_OWNER_ADDRESS` and `PLATFORM_OPERATOR_ADDRESS` (from `.env`). This is the only step that uses an Anvil default key, since it's just a funding transfer -- not a platform operation.
2. **`deploy-beacon`** -- Deploys the EVMAuth beacon implementation contract via the **beacon owner wallet** through the wallets service `/internal/send-tx` endpoint. Requires the wallets service to be running.
3. **`deploy-platform`** -- Deploys the platform proxy contract (pointing to `EVMAUTH_BEACON_ADDRESS` from `.env`) via the **platform operator wallet** through the wallets service.

These tasks can also be run directly via the CLI:

```bash
cargo run --package evmauth-cli -- fund $BEACON_OWNER_ADDRESS --amount 100
cargo run --package evmauth-cli -- fund $PLATFORM_OPERATOR_ADDRESS --amount 100
cargo run --package evmauth-cli -- deploy beacon
cargo run --package evmauth-cli -- deploy platform --beacon $EVMAUTH_BEACON_ADDRESS
```

Both wallets are Turnkey-managed HD wallets within the parent org. They are created once in the Turnkey console and their addresses are recorded in `.env` as static configuration values.

---

## 12. Production Deployment -- Railway

### Services

| Railway Service | Source | Notes |
|---|---|---|
| `evmauth-gateway` | `rs/` Dockerfile (gateway binary) | Entry point, PORT env var |
| `evmauth-auth` | `rs/` Dockerfile (auth binary) | Internal network only |
| `evmauth-wallets` | `rs/` Dockerfile (wallets binary) | Internal network only |
| `evmauth-registry` | `rs/` Dockerfile (registry binary) | Internal network only |
| `evmauth-analytics` | `rs/` Dockerfile (analytics binary) | Internal network only |
| `evmauth-assets` | `rs/` Dockerfile (assets binary) | Internal network only |
| `evmauth-docs` | `rs/` Dockerfile (docs binary) | Internal network only |
| `evmauth-console` | `ts/` Dockerfile (`SERVICE=console`) | Next.js, NODE_ENV=production |
| `evmauth-postgres` | Railway Postgres plugin | Managed, auto-backups |
| `evmauth-redis` | Railway Redis plugin | Managed |

### Backend Production Dockerfile (Existing)

The existing `rs/Dockerfile` is a multi-stage build that compiles all service binaries. Each Railway service selects its binary via the CMD override.

### Frontend Production Dockerfile (`ts/Dockerfile`)

A unified multi-stage Dockerfile at `ts/Dockerfile` (mirroring the `rs/Dockerfile` pattern) builds any TypeScript service via the `SERVICE` build arg. It has three stages: `dev` (Tilt hot reload), `builder` (production build), and `runtime` (standalone Next.js output).

```dockerfile
FROM node:22-alpine AS base
RUN corepack enable && corepack prepare pnpm@9.15.0 --activate
WORKDIR /workspace

FROM base AS dev
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml* ./
COPY packages/ ./packages/
ARG SERVICE
COPY services/${SERVICE}/package.json ./services/${SERVICE}/
RUN pnpm install || true
CMD ["sh", "-c", "pnpm --filter ${SERVICE} dev"]

FROM base AS builder
ARG SERVICE
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml ./
COPY packages/ ./packages/
COPY services/${SERVICE}/package.json ./services/${SERVICE}/
RUN pnpm install --frozen-lockfile
COPY services/${SERVICE}/ ./services/${SERVICE}/
RUN pnpm --filter ${SERVICE} run build

FROM node:22-alpine AS runtime
WORKDIR /app
ARG SERVICE
COPY --from=builder /workspace/services/${SERVICE}/.next/standalone ./
COPY --from=builder /workspace/services/${SERVICE}/.next/static ./.next/static
COPY --from=builder /workspace/services/${SERVICE}/public ./public
EXPOSE 3000
CMD ["node", "server.js"]
```

Enable `output: 'standalone'` in `next.config.ts`. The build context for Railway should be set to `ts/`. Each Railway frontend service sets the `SERVICE` build arg to select which service to build.

### Railway Config (railway.toml)

```toml
[build]
builder = "DOCKERFILE"

[deploy]
healthcheckPath = "/health"
healthcheckTimeout = 30
restartPolicyType = "ON_FAILURE"
```

### Migrations

Run as a Railway job (one-off) on each deploy before the backend services restart. The `db` service binary runs all pending migrations and exits.

---

## 13. Implementation Phases

### Phase 1 -- Foundation (COMPLETE)

- [x] Repository scaffolding: Cargo workspace with microservices architecture
- [x] Docker Compose: PostgreSQL 17 + pgvector, Redis 8, MinIO
- [x] Tiltfile: Auto-discovery, dynamic compose generation, hot reload, health checks
- [x] Database migrations: `auth.entities`, `auth.people`, `auth.orgs`, `auth.orgs_people`, `assets.*`
- [x] Database triggers: Personal workspace auto-creation, owner role sync, membership validation
- [x] Auth service bootstrap: Axum router, AppState (db + redis), health endpoint, error handling
- [x] People CRUD endpoints with pagination and filtering
- [x] Org CRUD endpoints with pagination and filtering
- [x] Org members management endpoints
- [x] Internal API variant (feature-gated `internal-api`)
- [x] Gateway service: Path-based routing, service discovery, error mapping
- [x] Docs service: OpenAPI aggregation, Swagger UI
- [x] Shared crates: postgres, redis-client, pagination, pagination-macros, service-discovery
- [x] Assets service scaffolding: S3 client, domain types, DTOs, repository stubs
- [x] Quality checks script (`check.sh`): biome check, tsc, fmt, clippy, test
- [x] Production Dockerfile (multi-stage build)
- [x] Turnkey integration: official SDK (`turnkey_client` + `turnkey_api_key_stamper`) used directly by wallets service (replaces custom wrapper crate)
- [x] Wallets service: scaffold, `wallets` schema migrations, org wallet + person sub-org + person app wallet internal APIs
- [x] Auth service: session JWT utilities, `RequireSession` middleware, auth code migration
- [x] Auth service: `GET /me`, `PATCH /me` endpoints (protected by RequireSession middleware)
- [x] Auth service: internal APIs for cross-service person/org lookup (`/internal/people/{id}`, `/internal/orgs/{id}`)
- [x] Frontend: PNPM workspace scaffolding (`ts/pnpm-workspace.yaml`, root `package.json`, `biome.json`, `tsconfig.json`)
- [x] Frontend: `@evmauth/ui` package (Mantine theme, ThemeProvider), `@evmauth/tsconfig` package (base + nextjs configs)
- [x] Frontend: Console service scaffolding (`ts/services/console/`), `service.json`, API proxy route
- [x] Frontend: Tiltfile TypeScript service auto-discovery (extend `discover_services` for `ts/services/`)
- [x] Docker init scripts for `registry` and `analytics` schemas
- [x] Workspace resolver set to v3 for edition 2024 compatibility
- [x] Service `.env.example` files: rewrite all with empty secrets, add missing vars (JWT, wallets URL); create wallets and console env files
- [x] Auth service: deployer signup/login (passkey + OAuth), HTTP-only cookie
- [x] EVM crate: Alloy HTTP provider, EVMAuth6909 bindings (balanceOf, isOperator, encode_mint)
- [x] Platform contract config (`PLATFORM_CONTRACT_ADDRESS`, `RADIUS_RPC_URL`, `RADIUS_CHAIN_ID`) in auth service
- [x] Capability token minting on new org creation (best-effort mint via wallets service `/internal/sign`)
- [x] Crate convention: shared crates accept config structs, never read environment variables directly
- [x] Frontend: Console login page, console shell, org overview page
- [x] Frontend: Code quality audit -- all 9 sections pass clean (TypeScript strictness, React correctness, API proxy security, session hardening, security, component architecture, Biome, App Router, workspace hygiene)
- [x] Frontend: `lib/config.ts` centralizes all `process.env` access with validation
- [x] Frontend: `lib/schemas.ts` shared Zod schemas for auth response validation
- [x] Frontend: API proxy reads iron-session, strips cookies, attaches `X-Person-Id`
- [x] Frontend: Auth routes validate request/response bodies with Zod, no Set-Cookie forwarding
- [x] Frontend: `skipLibCheck: false` with `@types/cookie` for full type checking
- [x] Frontend: `loading.tsx`, `error.tsx` boundaries for all route segments
- [x] Frontend: Metadata exports on all public pages
- [x] Frontend: All exported functions have explicit return types
- [x] Frontend: `useForm` from `@mantine/form` for login form validation

### Phase 2 -- App Registrations & Contracts

- [x] Registry service: scaffold, `registry` schema migrations
- [x] App registration domain, DTOs, repository, CRUD handlers
- [x] EVM crate: `encode_initialize`, `encode_grant_role`, `encode_revoke_role`, `encode_beacon_proxy_deploy` (BeaconProxy bytecode), `roles` module
- [x] Contract domain, DTOs, repository, handlers
- [x] Contract deployment endpoint with `initialize()` calldata (registry calls wallets internal API for wallet lookup)
- [x] Role grant/revoke endpoints (RESTful `/roles` resource) + Turnkey signing via wallets service
- [x] Relevant token ID configuration per app registration
- [x] Wallets service: `POST /internal/send-tx` endpoint (sign + broadcast via Turnkey + Alloy)
- [x] Registry internal API: `GET /internal/apps/by-client-id/{client_id}`, `GET /internal/contracts/{id}`
- [x] Rewrite wallets schema: `entity_wallets` table (replaces `org_wallets` + `person_turnkey_refs`)
- [x] Rewrite wallets schema: `entity_app_wallets` table (replaces `person_app_wallets`, adds org support)
- [x] EntityWallet domain, DTOs, repository in wallets service
- [x] EntityAppWallet domain, DTOs, repository in wallets service
- [x] Wallets service: `POST /internal/entity-wallet` endpoint
- [x] Wallets service: `POST /internal/entity-app-wallet` endpoint
- [x] Update auth signup flow to use `entity-wallet` endpoint
- [x] Update app registration creation to trigger entity app wallet derivation
- [x] Update contract deployment to use org's entity app wallet as EVMAuth default admin
- [x] Internal CLI tool (`evmauth-cli`) for beacon and platform contract deployment
- [x] Console: App registration pages, contract deployment wizard, role grant management UI

### Phase 3 -- End User Auth

- [ ] Hosted auth UI (`/auth/end-user/login`) with `client_id` / `redirect_uri` / PKCE parameter validation (auth calls registry internal API)
- [ ] End user sub-org creation on first login (auth calls wallets internal API)
- [ ] HD wallet account creation per (person, app_registration) via wallets service
- [ ] Authorization code generation, storage (hashed in Redis with TTL), and issuance
- [ ] PKCE token exchange endpoint with code_verifier validation, single-use enforcement (Redis DEL), and expiry check
- [ ] End-user app JWT issuance (auth calls wallets internal API for wallet address)
- [ ] JWKS endpoint (`GET /auth/.well-known/jwks.json`)
- [ ] End-user wallet self-service page (frontend calls wallets service via gateway)
- [ ] Passkey backup authenticator prompt on first login

### Phase 4 -- Authorization Query API

- [ ] ERC-712 request signing: define domain, type, and canonical digest
- [ ] `RequireErc712Auth` middleware in registry service
- [ ] `GET /registry/accounts/{address}` endpoint
- [ ] `balances_for` implementation in evm crate
- [ ] `is_operator` check for delegate flow
- [ ] Registry calls analytics internal API to log requests
- [ ] Deployer-facing SDK (TypeScript): wraps Turnkey delegated account signing into a single `client.accounts(address, contract)` call

### Phase 5 -- Analytics & Collaboration

- [ ] Analytics service: scaffold, `analytics` schema migrations
- [ ] API request logging (internal endpoint called by registry)
- [ ] Contract event indexing (background task polling chain)
- [ ] Console: Analytics pages (usage, events)
- [ ] Org member invite flow (email invitation, includes Turnkey sub-org user creation)
- [ ] Wallets service: `POST /internal/org-sub-org-user` endpoint (add user to org's Turnkey sub-org)
- [ ] `RequireOrgRole` middleware enforcement on org-scoped routes (auth + registry)
- [ ] Org settings page
- [ ] Turnkey org sub-org policy configuration

### Phase 6 -- Polish & Production

- [ ] Railway deployment configuration, staging environment
- [ ] Railway service setup (all services + managed postgres + redis)
- [ ] Migration job in Railway pipeline
- [ ] Rate limiting on `/registry/accounts` endpoint (`tower` governor layer)
- [ ] Structured logging (JSON, tracing spans with request IDs)
- [ ] Complete assets service implementation (file upload for company logos, etc.)
- [ ] Deployer-facing SDK documentation and quickstart guide

---

## 14. Environment Variables Reference

```bash
# ---- Infrastructure (Existing in .env.example) ----

POSTGRES_USER=db_admin
POSTGRES_PASSWORD=db_admin_password
POSTGRES_PORT=5432
POSTGRES_DB=evmauth

REDIS_PASSWORD=redis_password

MINIO_ROOT_USER=minio_admin
MINIO_ROOT_PASSWORD=minio_password

# ---- Backend Services (shared) ----

# Database (per service -- all share one Postgres instance, different schemas)
POSTGRES_HOST=postgres
POSTGRES_MAX_CONNECTIONS=10
POSTGRES_MIN_CONNECTIONS=2

# Redis (per service)
REDIS_HOST=redis
REDIS_PORT=6379

# Server (per service, default 8000)
PORT=8000

# ---- Auth Service ----

# JWT signing (RS256 -- generate with: openssl genrsa -out private.pem 2048)
JWT_PRIVATE_KEY_PEM=...
JWT_PUBLIC_KEY_PEM=...
AUTH_CODE_TTL_SECS=30
RUST_LOG=info,auth=debug

# ---- Wallets Service ----

# Turnkey
TURNKEY_API_BASE_URL=https://api.turnkey.com
TURNKEY_PARENT_ORG_ID=...
TURNKEY_API_PUBLIC_KEY=...
TURNKEY_API_PRIVATE_KEY=...

# Beacon owner wallet (Turnkey-managed -- high-privilege, rarely used)
BEACON_OWNER_TURNKEY_WALLET_ID=...
BEACON_OWNER_ADDRESS=0x...

RUST_LOG=info,wallets=debug

# ---- Registry Service ----

# Platform operator wallet (Turnkey-managed -- no raw private key)
PLATFORM_OPERATOR_TURNKEY_WALLET_ID=...
PLATFORM_OPERATOR_ADDRESS=0x...

# Platform EVMAuth contract (the platform's own deployed proxy)
PLATFORM_CONTRACT_ADDRESS=0x...

# Radius Network (mainnet: chain 723, testnet: chain 72344, local Anvil: chain 31337)
RADIUS_RPC_URL=https://rpc.radiustech.xyz          # Mainnet; testnet: https://rpc.testnet.radiustech.xyz; local: http://localhost:8545
RADIUS_CHAIN_ID=723                                 # Mainnet; testnet: 72344; local: 31337
EVMAUTH_BEACON_ADDRESS=0x...
RUST_LOG=info,registry=debug

# ---- Assets Service ----

# S3/MinIO
S3_ENDPOINT=http://minio:9000
S3_ACCESS_KEY=minio_admin
S3_SECRET_KEY=minio_password
S3_BUCKET=evmauth
S3_REGION=us-east-1

# ---- Gateway ----

API_GATEWAY_URL=https://api.evmauth.com
GATEWAY_TIMEOUT_SECS=30
EXCLUDE_SERVICES=gateway,db

# ---- Frontend: Console (ts/services/console) ----

# Server-side only (not NEXT_PUBLIC_)
BACKEND_URL=http://gateway:8000      # Internal gateway URL (never exposed to browser)
SESSION_SECRET=...                    # 32+ chars, required (no hardcoded fallback)
```

---

## Notes for Claude Code

- Shared crates (`rs/crates/`) must never read environment variables or call `dotenvy`. They accept plain config structs; the consuming service is responsible for populating them from env vars or any other source.
- Always run `sqlx prepare` after changing queries to keep the offline query cache (`.sqlx/`) in sync. Use `tilt trigger sqlx-prepare` in local dev.
- Use `sqlx::query_as!` macros for all DB queries -- no string-interpolated SQL.
- The `evm` crate should be integration-tested against a local Anvil fork of Radius, not a mock. Add an Anvil service to docker-compose for tests.
- Turnkey API calls use the official `turnkey_client` SDK which handles retry logic internally via `RetryConfig` (exponential backoff, default 5 attempts). Do not add custom retry wrappers.
- The platform has two Turnkey-managed HD wallets in the parent org: the **beacon owner wallet** (high-privilege, owns the UpgradeableBeacon and holds admin roles on the platform proxy) and the **platform operator wallet** (routine operations -- deploys proxies, mints/burns capability tokens). All signing goes through the Turnkey SDK via the wallets service. The `evm` crate is read-only (no signer); services encode calldata via `EvmClient::encode_mint()` and POST to the wallets service `/internal/sign` endpoint. No raw private keys in any environment variable.
- All contract addresses, tx hashes, and wallet addresses are stored as `TEXT` in Postgres (not `BYTEA`), EIP-55 checksummed format. Normalise on insert.
- The `/auth/end-user/login` page must validate `redirect_uri` against `callback_urls` **before** initiating the OAuth flow -- never after -- to prevent open redirect attacks. Auth service calls registry internal API to validate.
- Authorization codes are stored as `SHA-256(plaintext_code)` in Redis with key `auth_code:{hash}` and configurable TTL (default 30s via `AUTH_CODE_TTL_SECS`). The plaintext is returned once in the redirect and never stored. On token exchange, hash the submitted code and look it up in Redis, then delete immediately -- never store or log the plaintext code.
- The ERC-712 nonce is a Unix timestamp in seconds. Reject requests where `abs(now - nonce) > 30`. This is replay protection, not a monotonic counter.
- The Turnkey delegated account policy must be set at org sub-org creation time and must restrict to `ACTIVITY_TYPE_SIGN_RAW_PAYLOAD` only. The wallets service owns this concern.
- Session cookies must be: `HttpOnly`, `Secure`, `SameSite=Strict`.
- The platform EVMAuth contract must be deployed and its address recorded in config before any org can be created. The registry service should verify `PLATFORM_CONTRACT_ADDRESS` is reachable on Radius at startup.
- Each service only reads/writes its own schema. Cross-service data access goes through internal APIs. Never add FK constraints across schemas.
- New services should follow the established patterns: `service.json` for Tilt metadata, `api/error.rs` for `ApiError`, repository traits, utoipa annotations, health check endpoint.
- The gateway auto-discovers services -- just add a new service directory under `rs/services/` and the Tiltfile picks it up.
- TypeScript services follow the same auto-discovery pattern: add a directory under `ts/services/` with a `package.json` and `service.json` and the Tiltfile picks it up.
- Use PNPM workspace protocol (`"workspace:*"`) for all internal package references in `ts/`. Never publish shared packages to npm -- they are workspace-only.
- The `ts/packages/ui` package owns the Mantine theme. All services import the theme from `@evmauth/ui` -- never duplicate theme configuration.
- Use the existing `check.sh` script for quality checks. It runs TypeScript checks (`biome check` + `pnpm -r run typecheck`) at the `ts/` workspace root, then Rust checks (`cargo fmt --check`, `cargo clippy --workspace`, `cargo test --workspace`) at the `rs/` workspace root.
- Each entity (person or org) has one HD wallet in its Turnkey sub-org via `entity_wallets`. Account index 0 is the entity's platform identity. Additional accounts are derived per app and stored in `entity_app_wallets`.
- Contract deployment must use the org's app-specific derived address as EVMAuth default admin, not the org's platform identity address (index 0).
- When creating an app registration, the registry service must call wallets to derive the entity app wallet before returning success.
