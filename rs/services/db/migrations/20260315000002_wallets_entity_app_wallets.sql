-- Entity app wallet: HD wallet account derived for a specific (entity, app) pair.
-- For orgs, this derived address becomes the EVMAuth default admin for the app's
-- proxy contract. For people, this is the end user's identity within that app.
CREATE TABLE wallets.entity_app_wallets (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id               UUID NOT NULL,
    app_registration_id     UUID NOT NULL,
    wallet_address          TEXT NOT NULL UNIQUE,
    turnkey_account_id      TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (entity_id, app_registration_id)
);
CREATE INDEX idx_entity_app_wallets_entity_id ON wallets.entity_app_wallets(entity_id);
CREATE INDEX idx_entity_app_wallets_address ON wallets.entity_app_wallets(wallet_address);
CREATE INDEX idx_entity_app_wallets_pagination ON wallets.entity_app_wallets(created_at, id);

CREATE TRIGGER but_entity_app_wallets_moddatetime
    BEFORE UPDATE ON wallets.entity_app_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
