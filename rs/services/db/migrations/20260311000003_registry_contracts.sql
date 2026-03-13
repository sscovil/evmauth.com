-- Deployed EVMAuth proxy contracts on Radius.
CREATE TABLE registry.contracts (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  UUID NOT NULL,          -- references auth.orgs (no FK)
    app_registration_id     UUID,                   -- references registry.app_registrations (nullable)
    name                    TEXT NOT NULL,
    address                 TEXT NOT NULL UNIQUE,    -- on-chain address (EIP-55 checksummed)
    chain_id                TEXT NOT NULL,
    beacon_address          TEXT NOT NULL,
    deploy_tx_hash          TEXT NOT NULL,
    deployed_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_contracts_org_id ON registry.contracts(org_id);
CREATE INDEX idx_contracts_pagination ON registry.contracts(created_at, id);

CREATE TRIGGER but_contracts_moddatetime
    BEFORE UPDATE ON registry.contracts
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);

-- Platform operator grants: deployer has called setOperator on their contract
-- granting the platform operator wallet mint/burn access.
CREATE TABLE registry.operator_grants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id     UUID NOT NULL REFERENCES registry.contracts(id) ON DELETE CASCADE,
    grant_tx_hash   TEXT NOT NULL,
    revoke_tx_hash  TEXT,
    active          BOOLEAN NOT NULL DEFAULT true,
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_operator_grants_contract_id ON registry.operator_grants(contract_id);

CREATE TRIGGER but_operator_grants_moddatetime
    BEFORE UPDATE ON registry.operator_grants
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
