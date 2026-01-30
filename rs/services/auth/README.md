# Auth Service

REST API for managing people, organizations, and memberships.

## Data Model

### Entities

Base polymorphic table for all entities in the system. Supports querying across both people and organizations.

### People

User accounts with authentication provider integration.

- `display_name`: User's display name
- `description`: Optional user bio/description
- `auth_provider_name`: OAuth provider (e.g. "turnkey", "privy")
- `auth_provider_ref`: Provider's user ID reference
- `primary_email`: User's primary email (unique)

### Organizations

Groups that can have members with defined roles.

- `display_name`: Organization name
- `description`: Optional organization description
- `owner_id`: UUID of the owning person
- `visibility`: Whether the org is `private`, `public`, or a user's `personal` workspace

A `personal` workspace can only have one member, and that member must be the owner.
Every user must have at least one `personal` workspace.

### Organization Members

Join table linking people to organizations with roles.

- `org_id`: Organization UUID
- `member_id`: Person UUID
- `role`: Member role string (e.g. "admin", "member")

Constraints:
- Only the owner can be a member of their default org
- A person can only be a member of one default org

## Configuration

- `POSTGRES_DB` - Database name
- `POSTGRES_HOST` - Database host
- `POSTGRES_MAX_CONNECTIONS` - Connection pool max size (default: 5)
- `POSTGRES_MIN_CONNECTIONS` - Connection pool min size (default: 0)
- `POSTGRES_PORT` - Database port (default: 5432)
- `POSTGRES_PASSWORD` - Database password
- `POSTGRES_USER` - Database user

- `REDIS_HOST` - Redis host
- `REDIS_PASSWORD` - Redis password
- `REDIS_PORT` - Redis port (default: 6379)

## Development

After modifying SQL queries, update query metadata for offline compilation by triggering the **sqlx-prepare** resource in the [Tilt UI](http://localhost:10350/), or by running this command from the repository root:

```bash
tilt trigger sqlx-prepare
```

This updates the files in the `rs/.sqlx/` directory, which should be committed to version control.

## Running

```bash
cargo run -p auth
```

Run checks (format, lint, test, build) from the repository root:

```bash
./check.sh rs/services/auth
```
