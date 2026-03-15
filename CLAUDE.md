# EVMAuth Development Guidelines

## Project Overview

EVMAuth is a microservices platform built with Rust. The architecture consists of independently deployable services communicating through an API gateway.

## Technology Stack

- **Language**: Rust (Edition 2024, MSRV 1.88)
- **Web Framework**: Axum 0.8 with Tower-HTTP
- **Async Runtime**: Tokio
- **Database**: PostgreSQL with pgvector extension
- **Cache**: Redis
- **ORM**: SQLx with compile-time checked queries
- **API Docs**: utoipa (OpenAPI 3.1)
- **Development**: Tilt for local orchestration

## Directory Structure

```
rs/
├── services/          # Microservices
│   ├── auth/          # Authentication/user management
│   ├── wallets/       # Wallet lifecycle & Turnkey management (uses official SDK)
│   ├── registry/      # App registrations, contracts, accounts query
│   ├── gateway/       # API gateway (entry point)
│   ├── db/            # Database migrations
│   ├── docs/          # OpenAPI aggregation
│   └── assets/        # User uploaded file management
├── crates/            # Shared libraries
│   ├── evm/           # Alloy-based EVM interaction
│   ├── pagination/
│   ├── pagination-macros/
│   ├── postgres/
│   ├── redis-client/
│   └── service-discovery/
└── Cargo.toml         # Workspace configuration
```

### Service Internal Structure

```
service/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/mod.rs
│   ├── api/
│   │   ├── error.rs
│   │   ├── routes.rs
│   │   ├── openapi.rs
│   │   └── handlers/
│   ├── domain/
│   ├── dto/
│   │   ├── request/
│   │   └── response/
│   └── repository/
├── migrations/
└── seeds/
```

## Architecture Patterns

### Repository Pattern

Define trait abstractions for data access:

```rust
#[async_trait]
pub trait PersonRepository: Send + Sync {
    async fn create(&self, item: CreatePerson) -> Result<Person, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Option<Person>, RepositoryError>;
    async fn list(&self, filter: PersonFilter, page: Page) -> Result<Vec<Person>, RepositoryError>;
    async fn update(&self, id: Uuid, update: UpdatePerson) -> Result<Person, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}
```

Implementation naming: `PersonRepositoryImpl`

### Handler Pattern

```rust
#[utoipa::path(...)]
pub async fn handler_name(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResponseType>, ApiError> {
    let repo = RepositoryImpl::new(&state.db);
    let result = repo.method().await?;
    Ok(Json(result.into()))
}
```

### DTO Pattern

- Request DTOs: `Create{Entity}`, `Update{Entity}`
- Response DTOs: `{Entity}Response`
- Implement `From<DomainType> for ResponseType` for transformations

### Configuration Pattern

```rust
impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        // Load from environment with defaults
    }
}
```

## Code Conventions

### Naming

| Item            | Convention            | Example                |
|-----------------|-----------------------|------------------------|
| Services        | snake_case            | `auth`, `gateway`      |
| Modules         | snake_case            | `repository`, `api`    |
| Types/Structs   | PascalCase            | `PersonResponse`       |
| Functions       | snake_case            | `get_by_id`            |
| Constants       | SCREAMING_SNAKE_CASE  | `MAX_CONNECTIONS`      |
| Traits          | PascalCase            | `PersonRepository`     |
| Implementations | PascalCase + Impl     | `PersonRepositoryImpl` |

### SQL Naming

| Item               | Convention               | Example                  |
|--------------------|--------------------------|--------------------------|
| Tables             | plural snake_case        | `people`, `orgs`         |
| Join tables        | `{table1}_{table2}`      | `orgs_people`            |
| Indexes            | `idx_{table}_{columns}`  | `idx_people_email`       |
| Foreign keys       | `fk_{table}_{ref}`       | `fk_orgs_people_org`     |
| Primary keys       | `pk_{table}`             | `pk_people`              |
| Unique constraints | `uq_{table}_{columns}`   | `uq_people_email`        |
| Triggers           | `{type}_{table}_{action}` | `but_people_moddatetime` |

### Database Patterns

- Use UUIDs as primary keys via `gen_random_uuid()`
- Include `created_at` and `updated_at` timestamps on all tables
- Use `moddatetime` trigger for automatic `updated_at` updates
- Define schemas per service for isolation

## Error Handling

- Use `thiserror` for custom error definitions
- Implement `IntoResponse` for HTTP error conversion
- Map repository errors to API errors at the handler level
- Return unified JSON error responses

## Pagination

Use cursor-based pagination following Relay spec:

- Parameters: `first`, `after`, `last`, `before`
- Implement `Pageable` trait for cursor extraction
- Wrap responses in `PaginatedResponse<T>`

## Testing

- Write unit tests in same module using `#[cfg(test)]`
- Run checks with `./check.sh`:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`

## Development Workflow

1. Start infrastructure: `docker-compose up -d`
2. Run services: `tilt up`
3. Services auto-reload on file changes via cargo watch

## API Documentation

- Annotate all handlers with `#[utoipa::path(...)]`
- Use `#[derive(ToSchema)]` on DTOs
- Include `#[schema(...)]` attributes for field documentation
- Aggregated docs available via the docs service

## Key Principles

1. **Type Safety**: Use SQLx compile-time checked queries
2. **Modularity**: Separate domain, DTO, API, and repository layers
3. **Consistency**: Follow established patterns across all services
4. **Documentation**: Document public APIs with OpenAPI annotations
5. **Error Handling**: Use Result types, avoid panics
6. **Configuration**: Load from environment with sensible defaults
7. **Validation**: Validate at system boundaries (user input, external APIs)

## CRITICALLY IMPORTANT RULES

1. NEVER use `unwrap()` in Rust; it does not handle exceptions
2. NEVER use emoji anywhere in code or documentation
3. NEVER reimplement the same logic twice
4. ALWAYS think from first principles and use the cleanest, most elegant solution available
5. ALWAYS prefer code quality, performance, maintainability, and usability over backward compatability
