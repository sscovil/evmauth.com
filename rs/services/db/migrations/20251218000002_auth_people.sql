--- People table inheriting from entities
CREATE TABLE IF NOT EXISTS auth.people (
    auth_provider_name TEXT NOT NULL,
    auth_provider_ref TEXT NOT NULL,
    primary_email TEXT NOT NULL
) INHERITS (auth.entities);
ALTER TABLE auth.people ADD PRIMARY KEY (id);
ALTER TABLE auth.people ADD CONSTRAINT uq_people_email_provider UNIQUE (primary_email, auth_provider_name);
CREATE INDEX IF NOT EXISTS idx_people_pagination ON auth.people (created_at, id);

COMMENT ON COLUMN auth.people.auth_provider_name IS 'The name of the authentication provider (e.g. turnkey, privy).';
COMMENT ON COLUMN auth.people.auth_provider_ref IS 'The user ID provided by the authentication provider.';

CREATE TRIGGER but_people_moddatetime
    BEFORE UPDATE ON auth.people
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
