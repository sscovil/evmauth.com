-- Documents table inheriting from files
CREATE TABLE IF NOT EXISTS assets.docs () INHERITS (assets.files);
ALTER TABLE assets.docs ADD PRIMARY KEY (id);
ALTER TABLE assets.docs ADD FOREIGN KEY (org_id) REFERENCES auth.orgs (id) ON DELETE SET NULL;
ALTER TABLE assets.docs ADD FOREIGN KEY (uploader_id) REFERENCES auth.people (id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_docs_pagination ON assets.docs (created_at, id);

CREATE TRIGGER but_docs_moddatetime
    BEFORE UPDATE ON assets.docs
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
