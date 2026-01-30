-- Images table inheriting from files
CREATE TABLE IF NOT EXISTS assets.images (
    height INT NOT NULL,
    width INT NOT NULL
) INHERITS (assets.files);
ALTER TABLE assets.images ADD PRIMARY KEY (id);
ALTER TABLE assets.images ADD FOREIGN KEY (org_id) REFERENCES auth.orgs (id) ON DELETE SET NULL;
ALTER TABLE assets.images ADD FOREIGN KEY (uploader_id) REFERENCES auth.people (id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_images_pagination ON assets.images (created_at, id);

CREATE TRIGGER but_images_moddatetime
    BEFORE UPDATE ON assets.images
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
