-- Create analytics schema
CREATE SCHEMA analytics;

-- Create analytics role (non-login, for granting)
CREATE ROLE analytics_role NOLOGIN;
GRANT USAGE ON SCHEMA analytics TO analytics_role;
GRANT USAGE, CREATE ON SCHEMA analytics TO db_migrator;

-- Ensure objects created by db_migrator are accessible to analytics_role
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA analytics
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO analytics_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA analytics
    GRANT USAGE, SELECT ON SEQUENCES TO analytics_role;
ALTER DEFAULT PRIVILEGES FOR ROLE db_migrator IN SCHEMA analytics
    GRANT EXECUTE ON FUNCTIONS TO analytics_role;

-- Create login user for analytics service
CREATE USER analytics_user WITH PASSWORD 'analytics_password';
GRANT CONNECT ON DATABASE evmauth TO analytics_user;
GRANT analytics_role TO analytics_user;

-- Set default search path
ALTER ROLE analytics_user SET search_path = analytics;
ALTER USER db_migrator SET search_path = public, auth, assets, wallets, registry, analytics;
