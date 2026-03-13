# EVMAuth

Authorization state management built on the ERC-6909 multi-token standard, deployed on Radius Network.

## Prerequisites

- [Rust](https://rustup.rs/) (1.88+)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Tilt](https://tilt.dev/)
- [Node.js](https://nodejs.org/) (22+) and [pnpm](https://pnpm.io/)
- [Foundry](https://getfoundry.sh/) (forge, cast, anvil)

## Local Development

### 1. Start infrastructure

```bash
docker-compose up -d
```

This starts PostgreSQL, Redis, and MinIO.

### 2. Start Anvil (Radius fork)

Start a local Anvil instance forking Radius Network. Pin the block number for deterministic contract addresses:

```bash
anvil --fork-url https://rpc.radiustech.xyz --fork-block-number 1773253813276
```

### 3. Deploy EVMAuth contracts

From the EVMAuth contracts repository, deploy the beacon and platform proxy to the local Anvil fork.

**Deploy the beacon (one-time):**

```bash
forge script script/ExampleDeploy.s.sol:DeployBeacon6909 \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

**Deploy the platform proxy:**

```bash
BEACON=0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9 \
forge script script/ExampleDeploy.s.sol:DeployExampleProxy6909 \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

### Local contract addresses

With the pinned fork block number above, the deployment scripts produce deterministic addresses:

| Contract | Address |
|---|---|
| EVMAuth implementation | `0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0` |
| Beacon | `0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9` |
| Platform proxy (EVMAuth) | `0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9` |

### Local contract roles

The `DeployExampleProxy6909` script assigns roles to default Anvil accounts. These are derived from a well-known mnemonic and must never be used in production.

| Role | Anvil Account | Address |
|---|---|---|
| Deployer | #0 | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` |
| Default Admin | #1 | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` |
| Treasury | #2 | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` |
| Access Manager | #3 | `0x90F79bf6EB2c4f870365E785982E1f101E93b906` |
| Token Manager | #4 | `0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65` |
| Minter | #5 | `0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc` |
| Burner | #6 | `0x976EA74026E726554dB657fA54763abd0C3a0aa9` |
| Treasurer | #7 | `0x14dC79964da2C08b23698B3D3cc7Ca32193d9955` |

### 4. Start services

```bash
tilt up
```

Tilt auto-discovers all services in `rs/services/` and `ts/services/` and starts them with hot reload.

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
│   └── crates/          # Shared libraries (evm, pagination, postgres, redis-client, etc.)
├── ts/                  # TypeScript frontend (PNPM workspace)
│   ├── services/        # Next.js apps (dashboard)
│   └── packages/        # Shared packages (ui, tsconfig)
└── contracts/           # Solidity ABIs
```

See `PROJECT_PLAN.md` for detailed architecture documentation.
