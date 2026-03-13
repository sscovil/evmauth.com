-- App registrations: one per application using EVMAuth.
-- No client secret -- access is controlled by ERC-6909 token holdings
-- on the platform's own EVMAuth contract.
CREATE TABLE registry.app_registrations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id              UUID NOT NULL,              -- references auth.orgs (no FK)
    name                TEXT NOT NULL,
    client_id           TEXT NOT NULL UNIQUE,        -- public lookup key (random, URL-safe)
    callback_urls       TEXT[] NOT NULL DEFAULT '{}',
    relevant_token_ids  BIGINT[] NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_app_registrations_org_id ON registry.app_registrations(org_id);
CREATE INDEX idx_app_registrations_pagination ON registry.app_registrations(created_at, id);

CREATE TRIGGER but_app_registrations_moddatetime
    BEFORE UPDATE ON registry.app_registrations
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
