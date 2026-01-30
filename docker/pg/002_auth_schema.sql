-- Create auth schema
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
CREATE USER auth_user WITH PASSWORD 'auth_password';
GRANT CONNECT ON DATABASE evmauth TO auth_user;
GRANT auth_role TO auth_user;

-- Set default search path (optional but recommended)
ALTER ROLE auth_user SET search_path = auth;
ALTER USER db_migrator SET search_path = public, auth;
