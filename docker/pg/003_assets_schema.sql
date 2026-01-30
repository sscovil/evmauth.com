-- Create assets schema
CREATE SCHEMA assets;

-- Create assets role (non-login, for granting)
CREATE ROLE assets_role NOLOGIN;
GRANT USAGE ON SCHEMA assets TO assets_role;
GRANT USAGE, CREATE ON SCHEMA assets TO db_migrator;

-- Ensure objects created by db_migrator are accessible to assets_role
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA assets
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO assets_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA assets
    GRANT USAGE, SELECT ON SEQUENCES TO assets_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA assets
    GRANT EXECUTE ON FUNCTIONS TO assets_role;

-- Create login user for assets service
CREATE USER assets_user WITH PASSWORD 'assets_password';
GRANT CONNECT ON DATABASE evmauth TO assets_user;
GRANT assets_role TO assets_user;

-- Set default search path (optional but recommended)
ALTER ROLE assets_user SET search_path = assets;
ALTER USER db_migrator SET search_path = public, auth, assets;
