-- Entity wallet: Turnkey sub-org and HD wallet for any entity (person or org).
-- One per entity. Account index 0 is the entity's platform identity address.
-- For orgs, the delegated account holds a P256 API key scoped by Turnkey policy
-- to ACTIVITY_TYPE_SIGN_RAW_PAYLOAD only.
CREATE TABLE wallets.entity_wallets (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id                   UUID NOT NULL UNIQUE,
    turnkey_sub_org_id          TEXT NOT NULL UNIQUE,
    turnkey_wallet_id           TEXT NOT NULL,
    wallet_address              TEXT NOT NULL,
    turnkey_delegated_user_id   TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_entity_wallets_pagination ON wallets.entity_wallets(created_at, id);

CREATE TRIGGER but_entity_wallets_moddatetime
    BEFORE UPDATE ON wallets.entity_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
