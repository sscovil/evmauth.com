-- Org wallet: Turnkey sub-org, wallet address, and delegated account for an org.
-- One per org. The delegated account holds a server-side API key scoped by
-- Turnkey policy to ERC-712 signing only.
CREATE TABLE wallets.org_wallets (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                      UUID NOT NULL UNIQUE,
    turnkey_sub_org_id          TEXT NOT NULL UNIQUE,
    wallet_address              TEXT NOT NULL,
    turnkey_delegated_user_id   TEXT NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_org_wallets_pagination ON wallets.org_wallets(created_at, id);

CREATE TRIGGER but_org_wallets_moddatetime
    BEFORE UPDATE ON wallets.org_wallets
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
