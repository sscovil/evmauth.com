# EVMAuth -- Project Summary

## What It Is

EVMAuth.com is an authorization-as-a-service platform built on the ERC-6909 multi-token standard. It lets application developers manage user permissions entirely on-chain, replacing traditional role/permission databases with smart contract state on the Radius Network.

## The Problem

Authorization systems are typically tightly coupled to application backends -- hardcoded roles, permission tables, secret-based API keys. They're difficult to audit, impossible to share across services, and require manual rotation when access changes. Revoking access means updating databases and hoping nothing was cached.

## The Solution

EVMAuth moves authorization state on-chain. Each application gets a dedicated smart contract where token balances represent permissions. Granting access is a mint; revoking access is a burn. There are no secrets to rotate and no database records to invalidate. Authorization state is publicly verifiable, instantly revocable, and shared across any service that can read the chain.

The platform handles the complexity so developers don't have to:

- **Contract deployment**: One-click beacon proxy deployment. All contracts share a single upgradeable implementation, so the platform can ship improvements to every customer simultaneously.
- **Wallet management**: Turnkey-powered custodial wallets with hardware-level key isolation. Developers and end users get wallets automatically -- no MetaMask required.
- **Authentication**: A full PKCE-based OAuth flow for end users. Developers integrate with a standard authorization code exchange and receive signed JWTs containing wallet addresses.
- **Runtime authorization queries**: A single API endpoint (`GET /accounts/{address}`) returns live on-chain token balances. Requests are authenticated via ERC-712 signatures -- the caller's ability to query is itself an on-chain permission.

## Architecture

Rust microservices backend (Axum), Next.js frontend (Mantine UI), PostgreSQL, Redis. Services communicate through an API gateway with schema-per-service isolation. Each service owns its domain: auth handles identity, wallets manages Turnkey sub-orgs, registry handles app registrations and contracts, analytics indexes on-chain events.

The platform uses its own EVMAuth contract for internal access control -- API access rights are ERC-6909 token holdings, not secrets.

## Current State

Foundation complete: authentication service with signup/login, organization management, deployer dashboard with session-protected routes. Turnkey integration for wallet lifecycle. Infrastructure fully orchestrated with Tilt and Docker Compose.

Next milestones: app registration and contract deployment flows, end-user PKCE auth, and the runtime authorization query API.

## Key Technical Decisions

- **ERC-6909 over ERC-1155**: Simpler interface, lower gas costs, native operator delegation without approval transactions.
- **Beacon proxy pattern**: One implementation contract, unlimited proxies. Atomic upgrades across all customers.
- **No client secrets**: Access to the authorization API is controlled by on-chain token holdings. Nothing to leak, nothing to rotate.
- **ERC-712 request signing**: API authentication uses typed structured data signatures, verifiable off-chain. Replay-resistant via timestamp nonce.
- **Turnkey custodial wallets**: Hardware-isolated keys with policy-scoped signing. Delegated accounts restrict platform signing to ERC-712 payloads only.
