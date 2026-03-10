-- Create registry schema
CREATE SCHEMA registry;

-- Create registry role (non-login, for granting)
CREATE ROLE registry_role NOLOGIN;
GRANT USAGE ON SCHEMA registry TO registry_role;
GRANT USAGE, CREATE ON SCHEMA registry TO db_migrator;

-- Ensure objects created by db_migrator are accessible to registry_role
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA registry
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO registry_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA registry
    GRANT USAGE, SELECT ON SEQUENCES TO registry_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA registry
    GRANT EXECUTE ON FUNCTIONS TO registry_role;

-- Create login user for registry service
CREATE USER registry_user WITH PASSWORD 'registry_password';
GRANT CONNECT ON DATABASE evmauth TO registry_user;
GRANT registry_role TO registry_user;

-- Set default search path
ALTER ROLE registry_user SET search_path = registry;
ALTER USER db_migrator SET search_path = public, auth, assets, wallets, registry;
