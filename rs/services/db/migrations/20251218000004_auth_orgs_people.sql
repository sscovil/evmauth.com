-- Junction table for many-to-many relationship between orgs and people
CREATE TABLE IF NOT EXISTS auth.orgs_people (
    org_id UUID NOT NULL REFERENCES auth.orgs (id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES auth.people (id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (org_id, member_id)
);
CREATE INDEX IF NOT EXISTS idx_orgs_people_member_id ON auth.orgs_people(member_id);
CREATE INDEX IF NOT EXISTS idx_orgs_people_pagination ON auth.orgs_people (created_at, member_id);

-- Ensure only one owner per org
CREATE UNIQUE INDEX IF NOT EXISTS idx_orgs_people_unique_owner
ON auth.orgs_people (org_id)
WHERE role = 'owner';

COMMENT ON COLUMN auth.orgs_people.role IS 'The role of the user within the organization (e.g. owner, admin, member).';

CREATE TRIGGER but_orgs_people_moddatetime
    BEFORE UPDATE ON auth.orgs_people
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
