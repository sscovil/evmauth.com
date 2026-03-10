-- Create wallets schema
CREATE SCHEMA wallets;

-- Create wallets role (non-login, for granting)
CREATE ROLE wallets_role NOLOGIN;
GRANT USAGE ON SCHEMA wallets TO wallets_role;
GRANT USAGE, CREATE ON SCHEMA wallets TO db_migrator;

-- Ensure objects created by db_migrator are accessible to wallets_role
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA wallets
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO wallets_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA wallets
    GRANT USAGE, SELECT ON SEQUENCES TO wallets_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA wallets
    GRANT EXECUTE ON FUNCTIONS TO wallets_role;

-- Create login user for wallets service
CREATE USER wallets_user WITH PASSWORD 'wallets_password';
GRANT CONNECT ON DATABASE evmauth TO wallets_user;
GRANT wallets_role TO wallets_user;

-- Set default search path
ALTER ROLE wallets_user SET search_path = wallets;
ALTER USER db_migrator SET search_path = public, auth, assets, wallets;
