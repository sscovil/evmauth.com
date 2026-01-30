-- Orgs table inheriting from entities
CREATE TABLE IF NOT EXISTS auth.orgs (
    owner_id UUID NOT NULL REFERENCES auth.people (id) ON DELETE RESTRICT,
    visibility TEXT NOT NULL DEFAULT 'private'
) INHERITS (auth.entities);
ALTER TABLE auth.orgs ADD PRIMARY KEY (id);
CREATE UNIQUE INDEX uq_org_owner_id_visibility_personal
    ON auth.orgs (owner_id)
    WHERE visibility = 'personal';
CREATE INDEX IF NOT EXISTS idx_orgs_pagination ON auth.orgs (created_at, id);

COMMENT ON COLUMN auth.orgs.owner_id IS 'The user who owns this organization or workspace.';
COMMENT ON COLUMN auth.orgs.visibility IS 'Visibility of the organization or workspace: personal, private, or public.';

CREATE TRIGGER but_orgs_moddatetime
    BEFORE UPDATE ON auth.orgs
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
