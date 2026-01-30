-- Media (audio/video) table inheriting from images, which inherits from files
CREATE TABLE IF NOT EXISTS assets.media (
    duration_ms INT NOT NULL
) INHERITS (assets.images);
ALTER TABLE assets.media ADD PRIMARY KEY (id);
ALTER TABLE assets.media ADD FOREIGN KEY (org_id) REFERENCES auth.orgs (id) ON DELETE SET NULL;
ALTER TABLE assets.media ADD FOREIGN KEY (uploader_id) REFERENCES auth.people (id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_media_pagination ON assets.media (created_at, id);

COMMENT ON COLUMN assets.media.duration_ms IS 'Length of the media file in milliseconds.';

CREATE TRIGGER but_media_moddatetime
    BEFORE UPDATE ON assets.media
    FOR EACH ROW
EXECUTE FUNCTION moddatetime(updated_at);
