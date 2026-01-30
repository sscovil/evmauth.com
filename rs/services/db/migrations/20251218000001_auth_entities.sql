-- Base entities table for all auth-related entities
CREATE TABLE IF NOT EXISTS auth.entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_entities_pagination ON auth.entities (created_at, id);

COMMENT ON COLUMN auth.entities.display_name IS 'The public name of the entity.';
COMMENT ON COLUMN auth.entities.description IS 'A public description of the entity.';

CREATE TRIGGER but_entities_moddatetime
    BEFORE UPDATE ON auth.entities
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
