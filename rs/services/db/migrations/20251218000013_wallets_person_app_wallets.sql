-- End-user wallet accounts: one per (person, app_registration) pair.
-- Created when an end user first authenticates with a deployer's app.
CREATE TABLE wallets.person_app_wallets (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id               UUID NOT NULL,
    app_registration_id     UUID NOT NULL,
    wallet_address          TEXT NOT NULL,
    turnkey_account_id      TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (person_id, app_registration_id),
    UNIQUE (wallet_address)
);
CREATE INDEX idx_person_app_wallets_person_id ON wallets.person_app_wallets(person_id);
CREATE INDEX idx_person_app_wallets_address ON wallets.person_app_wallets(wallet_address);
CREATE INDEX idx_person_app_wallets_pagination ON wallets.person_app_wallets(created_at, id);

CREATE TRIGGER but_person_app_wallets_moddatetime
    BEFORE UPDATE ON wallets.person_app_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
