-- Person's Turnkey sub-org reference. One per person.
CREATE TABLE wallets.person_turnkey_refs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id           UUID NOT NULL UNIQUE,
    turnkey_sub_org_id  TEXT NOT NULL UNIQUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_person_turnkey_refs_pagination ON wallets.person_turnkey_refs(created_at, id);

CREATE TRIGGER but_person_turnkey_refs_moddatetime
    BEFORE UPDATE ON wallets.person_turnkey_refs
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
