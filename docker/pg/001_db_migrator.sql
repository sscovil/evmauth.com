-- Lock down public schema (but allow db_migrator to use it for migrations)
REVOKE ALL ON SCHEMA public FROM PUBLIC;
REVOKE CONNECT ON DATABASE evmauth FROM PUBLIC;

-- Create db_migrator user
CREATE USER db_migrator WITH PASSWORD 'db_migrator_password';
ALTER USER db_migrator CREATEROLE;
ALTER USER db_migrator SET search_path = public;

-- Grants for db_migrator
GRANT CONNECT, CREATE ON DATABASE evmauth TO db_migrator;
GRANT USAGE, CREATE ON SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO db_migrator;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO db_migrator;
