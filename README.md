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

### 4. Fund the platform operator wallet

The platform operator wallet is managed by Turnkey, so Anvil default private keys cannot be used for deployment. Use the internal CLI tool to fund the operator address on local Anvil:

```bash
cargo run --package evmauth-cli -- fund <platform-operator-address> --amount 100
```

### 5. Deploy EVMAuth contracts

```bash
# Deploy the beacon implementation
cargo run --package evmauth-cli -- deploy beacon

# Deploy the platform proxy
cargo run --package evmauth-cli -- deploy platform --beacon <beacon-address>
```

### 6. Run checks

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
