# Database Manager Service

Database manager for PostgreSQL, used to run migrations (no REST API).

## Structure

```
migrations/  - Schema migrations (always run)
seeds/       - Development seed data (optional)
```

Migrations are applied in order based on their timestamp prefix (e.g. `20251218000001_*.sql`).

Seeds are only applied in your local dev environment (or, more precisely, when the `dev-seeds` feature is enabled).

## Configuration

- `POSTGRES_DB` - Database name
- `POSTGRES_HOST` - Database host
- `POSTGRES_MAX_CONNECTIONS` - Connection pool max size (default: 5)
- `POSTGRES_MIN_CONNECTIONS` - Connection pool min size (default: 0)
- `POSTGRES_PASSWORD` - Database password
- `POSTGRES_PORT` - Database port (default: 5432)
- `POSTGRES_USER` - Database user

## Running

Standard mode (migrations only):

```bash
cargo run -p db
```

Run checks (format, lint, test, build) from the repository root:

```bash
./check.sh rs/services/db
```

Development mode (migrations + seeds):

```bash
cargo run -p db --features dev-seeds
```

The `dev-seeds` feature runs seed data from the `seeds/` directory after migrations complete.

### Important Note on Seeds

We keep seeds out of the `migrations/` directory to avoid accidentally applying them in staging/production environments. However, the SQLx migration system expects all migration files to be in the same directory, and you cannot configure which table it uses to track applied migrations.

To get around this when the `dev-seeds` feature is enabled, we use two different instances of `sqlx::migrate::Migrator` pointing at different directories, with `set_ignore_missing` set to `true`. This tells SQLx to ignore any migrations that are missing from the database's migration tracking table—which will always be the case when running migrations from two different directories.

- **Benefit:** We don't need to make our seeds idempotent, or write custom logic to track which seeds have already been applied.
- **Drawback:** If we change any migrations that have already been applied, we won't get a compile-time error locally.

We only set `set_ignore_missing` to `true` when the `dev-seeds` feature is enabled (i.e. in local dev), so if changes are made to any migrations that have already been applied in staging/production, we **will** get a compile-time error when we deploy. If that happens, we'll need to revert the changes to the migration files that were already applied—which is fine, because they should never be changed anyway.

## Adding Migrations

Create a new migration file in the `rs/services/db/migrations` directory, with a filename like:

```
20251218134601_create_users_table.sql
```

Use the format `<YYYY><MM><DD><HH><MM><SS>` for the timestamp prefix, to avoid collisions.

Migrations are applied in timestamp order when the service runs.

## Adding Seeds

Same as migrations, but in the `rs/services/db/seeds` directory. Use the same timestamp prefix format.

## SQL Guide

See [SQL Guide](./docs/SQL_GUIDE.md) for detailed conventions on writing SQL, naming triggers/functions, and designing the database schema.
