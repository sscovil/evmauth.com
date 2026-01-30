# SQL Guide

This document outlines the SQL coding standards, naming conventions, and schema design principles used in our database systems.

## PostgreSQL Schemas

Each backend service should have its own [PostgreSQL schema](https://www.postgresql.org/docs/current/ddl-schemas.html), as well as a dedicated user role with read/write permissions for that schema.

The `public` schema should be locked down and not used by any application data (unless absolutely necessary).

```sql
REVOKE ALL ON SCHEMA public FROM PUBLIC;
REVOKE CONNECT ON DATABASE evmauth FROM PUBLIC;
```

Services may be granted read-only access to tables in other schemas as needed, for things like foreign key references, but only one service should own and manipulate data for each schema.

The one exception is that we have a `db_migrator` user with permission to execute database migrations from a dedicated service that can be run independently of other service deployments.

```sql
CREATE USER db_migrator WITH PASSWORD 'some_super_secure_password';
ALTER USER db_migrator CREATEROLE;
ALTER USER db_migrator SET search_path = public;

GRANT USAGE, CREATE ON SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO db_migrator;

GRANT CONNECT, CREATE ON DATABASE evmauth TO db_migrator;
```

To create a schema and database for a new service, we would run SQL commands like this using the PostgreSQL super admin user:

```sql
CREATE SCHEMA auth;

-- Create auth role (non-login, for granting)
CREATE ROLE auth_role NOLOGIN;
GRANT USAGE ON SCHEMA auth TO auth_role;
GRANT USAGE, CREATE ON SCHEMA auth TO db_migrator;

-- Ensure objects created by db_migrator are accessible to auth_role
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA auth
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO auth_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA auth
    GRANT USAGE, SELECT ON SEQUENCES TO auth_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA auth
    GRANT EXECUTE ON FUNCTIONS TO auth_role;

-- Create login user for auth service
CREATE USER auth_user WITH PASSWORD 'another_super_secure_password';
GRANT CONNECT ON DATABASE evmauth TO auth_user;
GRANT auth_role TO auth_user;

-- Set default search path (db_migrator should have all schemas)
ALTER ROLE auth_user SET search_path = auth;
ALTER USER db_migrator SET search_path = public, auth;
```

## PostgreSQL Extensions

### pgvector

We should install PosgreSQL from the [pgvector/pgvector:pg18-trixie](https://hub.docker.com/layers/pgvector/pgvector/pg18-trixie/images/sha256-4ccba8d798ace910ed02139e603f9357c370387c284433d9b4ceaec6c35a4b1d) Docker image.

When adding vector embeddings to a table, we should use the Hierarchical Navigable Small World ([HNSW](https://github.com/pgvector/pgvector?tab=readme-ov-file#hnsw)) algorithm:

```sql
CREATE INDEX idx_my_table_my_column
ON my_schema.my_table
USING hnsw (my_column vector_cosine_ops);
```

[HNSW](https://github.com/pgvector/pgvector?tab=readme-ov-file#hnsw) is generally preferred over [IVFFlat](https://github.com/pgvector/pgvector?tab=readme-ov-file#ivfflat) because it has better query performance than IVFFlat (in terms of speed-recall tradeoff), but has slower build times and uses more memory. Also, an index can be created without any data in the table since there isn’t a training step like IVFFlat.

IVFFlat is a nearest-neighbor algorithm that requires data to be present before building the index, and also requires rebuilding the index when the data size grows above 1M rows.

Note that the operator used when querying vector embeddings will only use the corresponding index (if it exists):

**Operator classes in pgvector:**

| Operator class      | Distance metric | Query operator |
|---------------------|-----------------|----------------|
| `vector_l2_ops`     | Euclidean (L2)  | `<->`          |
| `vector_cosine_ops` | Cosine distance | `<=>`          |
| `vector_ip_ops`     | Inner product   | `<#>`          |

So, if you define the index using `vector_cosine_ops`, it will only be used for queries that use the `<=>` operator for cosine similarity.

### moddatetime

We should install the moddatetime extension that ships with PostgreSQL:

```sql
CREATE EXTENSION moddatetime;
```

…and use it in a `BEFORE UPDATE` trigger on each table:

```sql
CREATE TABLE IF NOT EXISTS auth.entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_entities_pagination ON auth.entities (created_at, id);

COMMENT ON COLUMN auth.entities.display_name IS 'The public name of the entity.';
COMMENT ON COLUMN auth.entities.description IS 'A public description of the entity.';

CREATE TRIGGER but_entities_moddatetime
    BEFORE UPDATE ON auth.entities
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
```

## Naming Conventions

### Table Names

Tables should be named using plural nouns to represent collections of entities.

Junction tables for many-to-many relationships should be named using the plural forms of both related tables, separated by an underscore.

Example table names:
- `people`
- `orgs`
- `orgs_people`

### Index Names

Indexes should be prefixed with `idx_`, followed by the table name and the column(s) being indexed.

Example index names:

- `idx_users_email` - Index on the `email` column of the `users` table.
- `idx_orders_created_at` - Index on the `created_at` column of the `orders` table.

### Constraint Names

Constraints should be prefixed with an abbreviation for the type of constraint:

- `pk_` - Primary Key
- `fk_` - Foreign Key
- `uq_` - Unique Constraint
- `chk_` - Check Constraint

Example constraint names:

- `pk_users_id` - Primary Key constraint on the `id` column of the `users` table.
- `fk_orders_user_id` - Foreign Key constraint on the `user_id` column of the `orders` table referencing the `users` table.
- `uq_users_email` - Unique constraint on the `email` column of the `users` table.
- `chk_products_price_positive` - Check constraint ensuring the `price` column of the `products` table is positive.

### Trigger Names

Triggers should be prefixed with an abbreviation for the type of trigger:

- `bit_` - Before Insert Trigger
- `but_` - Before Update Trigger
- `biut_` - Before Insert or Update Trigger
- `bud_` - Before Delete Trigger
- `ait_` - After Insert Trigger
- `aut_` - After Update Trigger
- `aiut_` - After Insert or Update Trigger
- `aud_` - After Delete Trigger

...and so on.

The trigger prefix should be followed by the table name and a description of what the trigger does, separated by underscores.

Example trigger names:

- `bit_users_set_created_at` - Before Insert Trigger on the `users` table that sets the `created_at` timestamp.
- `aut_orders_update_total` - After Update Trigger on the `orders` table that updates the total amount.

### Function Names

Functions that are only used by a specific trigger should be prefixed with `tfn_` and scoped to the schema where the trigger is defined.

For example, a trigger named `bit_users_set_created_at` in the `app` schema would have a corresponding function named `app.tfn_users_set_created_at`.

If a function is not tied to a specific trigger, it should be named according to its purpose without the `tfn_` prefix.

## Schema Design

IDs are UUIDs generated via `DEFAULT gen_random_uuid()`, except in junction tables where composite primary keys are used.

All tables include standard timestamp fields:
- `created_at` - Set automatically on insert via `DEFAULT now()`
- `updated_at` - Set automatically on insert via `DEFAULT now()`, and updated automatically on row modification via the `moddatetime` trigger

[PostgreSQL inheritance](https://www.postgresql.org/docs/current/ddl-inherit.html) is used where appropriate (see below). In PostgreSQL, a table can inherit from zero or more other tables, and a query can reference either all rows of a table or all rows of a table plus all of its descendant tables.

### When to Use Inheritance

Use inheritance when modeling true "is-a" relationships where subtypes share common attributes but also have type-specific columns.

Inheritance is appropriate when:

- Subtypes have meaningful type-specific columns with their own constraints (e.g., `NOT NULL` on columns that only exist for that type)
- Polymorphic queries across all subtypes are needed (querying the parent returns all child rows automatically)
- The set of subtypes is relatively stable and known at design time

Inheritance avoids the following anti-patterns:

- **Sparse tables** - A single table with a `type` column and many nullable columns, where most columns are `NULL` for most rows. This prevents type-specific `NOT NULL` constraints, bloats indexes, and obscures which columns belong to which type.
- **EAV (Entity-Attribute-Value)** - Storing attributes as key-value pairs sacrifices type safety, constraint enforcement, and query performance.
- **JSON blobs for variants** - Storing type-specific data in a JSON column bypasses schema enforcement and limits indexing and constraint options.
- **Disconnected tables** - Separate tables with no inheritance relationship require `UNION ALL` for cross-type queries and duplicate shared column definitions.

Do not use inheritance when:

- Partitioning data by time or value range (use [declarative partitioning](https://www.postgresql.org/docs/current/ddl-partitioning.html) instead)
- Foreign key references to or from the hierarchy are required (see limitations below)
- Subtypes are dynamic or user-configurable

### Important Note on Constraints and Inheritance

From the documentation:

> All check constraints and not-null constraints on a parent table are automatically inherited by its children, unless explicitly specified otherwise with NO INHERIT clauses. Other types of constraints (unique, primary key, and foreign key constraints) are not inherited.

This means that primary keys, unique constraints, and foreign key references must all be defined on each child table individually, even if the parent table has them.

Also:

> A serious limitation of the inheritance feature is that indexes (including unique constraints) and foreign key constraints only apply to single tables, not to their inheritance children. This is true on both the referencing and referenced sides of a foreign key constraint.

This means that, if you need a unique constraint or foreign key reference that applies across all child tables, you need to implement that logic via triggers or application code. You can't have another table reference a foreign key that points to a parent table and expect it to apply to all child tables.
