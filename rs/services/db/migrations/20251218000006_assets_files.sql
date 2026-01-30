-- Base files table for all file types saved in object storage
CREATE TABLE IF NOT EXISTS assets.files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES auth.orgs (id) ON DELETE SET NULL,
    uploader_id UUID REFERENCES auth.people (id) ON DELETE SET NULL,
    object_key TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_files_pagination ON assets.files (created_at, id);

COMMENT ON COLUMN assets.files.object_key IS 'The key/path of the file in the object storage system.';

CREATE TRIGGER but_files_moddatetime
    BEFORE UPDATE ON assets.files
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
