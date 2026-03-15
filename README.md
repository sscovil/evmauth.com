# EVMAuth

Authorization state management built on the ERC-6909 multi-token standard, deployed on Radius Network.

## Prerequisites

- [Rust](https://rustup.rs/) (1.88+)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Tilt](https://tilt.dev/)
- [Node.js](https://nodejs.org/) (22+) and [pnpm](https://pnpm.io/)
- [Foundry](https://getfoundry.sh/) (anvil)

## Local Development

### 1. Start infrastructure

```bash
docker-compose up -d
```

This starts PostgreSQL, Redis, and MinIO.

### 2. Start Anvil (Radius fork)

Start a local Anvil instance forking Radius Network:

```bash
anvil --fork-url https://rpc.radiustech.xyz --fork-block-number 1773253813276
```

### 3. Start services

```bash
tilt up
```

Tilt auto-discovers all services in `rs/services/` and `ts/services/` and starts them with hot reload.

### 4. Deploy EVMAuth contracts

The platform uses two Turnkey-managed wallets (beacon owner and platform operator), so Anvil default private keys cannot be used for contract deployment. The Tilt dashboard provides manual tasks for funding and deploying:

1. **Fund wallets** -- In Tilt, trigger the `fund-wallets` task. This sends 100 ETH from Anvil account #0 to both `BEACON_OWNER_ADDRESS` and `PLATFORM_OPERATOR_ADDRESS` (read from your `.env`). This is the only step that uses an Anvil default key, since it's just a funding transfer.

2. **Deploy beacon** -- Trigger the `deploy-beacon` task. Deploys the EVMAuth beacon implementation contract via the beacon owner wallet through the wallets service `/internal/send-tx` endpoint.

3. **Deploy platform proxy** -- Trigger the `deploy-platform` task. Deploys the platform proxy contract (pointing to the beacon) via the platform operator wallet through the wallets service.

These tasks can also be run directly:

```bash
cargo run --package evmauth-cli -- fund $BEACON_OWNER_ADDRESS --amount 100
cargo run --package evmauth-cli -- fund $PLATFORM_OPERATOR_ADDRESS --amount 100
cargo run --package evmauth-cli -- deploy beacon
cargo run --package evmauth-cli -- deploy platform --beacon <beacon-address>
```

Both wallets are Turnkey-managed HD wallets created once in the Turnkey console within the parent org. Their addresses are recorded in your `.env` file as static configuration values (see `PROJECT_PLAN.md` Section 14 for all environment variables).

### 5. Run checks

```bash
./check.sh
```

Runs `biome check`, `tsc --noEmit`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.

## Project Structure

```
evmauth.com/
├── rs/                  # Rust backend (Cargo workspace)
│   ├── services/        # Microservices (auth, wallets, registry, gateway, docs, db, assets)
│   ├── tools/           # Internal tools (evmauth-cli)
│   └── crates/          # Shared libraries (evm, pagination, postgres, redis-client, etc.)
├── ts/                  # TypeScript frontend (PNPM workspace)
│   ├── services/        # Next.js apps (console)
│   └── packages/        # Shared packages (ui, tsconfig)
└── contracts/           # Solidity ABIs
```

See `PROJECT_PLAN.md` for detailed architecture documentation.
