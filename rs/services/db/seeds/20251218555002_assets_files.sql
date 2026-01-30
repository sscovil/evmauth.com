-- Seed some document files for Acme Corp
WITH acme_org AS (
    SELECT id FROM auth.orgs WHERE display_name = 'Acme Corp'
), alice AS (
    SELECT id FROM auth.people WHERE primary_email = 'alice@example.com'
), bob AS (
    SELECT id FROM auth.people WHERE primary_email = 'bob@example.com'
)
INSERT INTO assets.docs (org_id, uploader_id, object_key, file_name, mime_type, size_bytes) VALUES
(
    (SELECT id FROM acme_org),
    (SELECT id FROM alice),
    'fbf3c75b-98ec-408e-9148-aeb51a98fffa',
    'doc_1.pdf',
    'application/pdf',
    204800
),
(
    (SELECT id FROM acme_org),
    (SELECT id FROM bob),
    'b02de416-12dd-44a4-85c4-782b5991c660',
    'doc_2.doc',
    'application/msword',
    204800
);

-- Seed some image files for Acme Corp
WITH acme_org AS (
    SELECT id FROM auth.orgs WHERE display_name = 'Acme Corp'
), alice AS (
    SELECT id FROM auth.people WHERE primary_email = 'alice@example.com'
), bob AS (
    SELECT id FROM auth.people WHERE primary_email = 'bob@example.com'
)
INSERT INTO assets.images (org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width) VALUES
(
    (SELECT id FROM acme_org),
    (SELECT id FROM alice),
    '1121aea0-9d68-400e-8670-489d0856eb40',
    'image_1.png',
    'image/png',
    102400,
    800,
    600
),
(
    (SELECT id FROM acme_org),
    (SELECT id FROM bob),
    'd728dfc6-b390-4a90-acd5-722704de5630',
    'image_2.jpg',
    'image/jpeg',
    153600,
    1024,
    768
);

-- Seed some audio/video files for Acme Corp
WITH acme_org AS (
    SELECT id FROM auth.orgs WHERE display_name = 'Acme Corp'
), alice AS (
    SELECT id FROM auth.people WHERE primary_email = 'alice@example.com'
), bob AS (
    SELECT id FROM auth.people WHERE primary_email = 'bob@example.com'
)
INSERT INTO assets.media (org_id, uploader_id, object_key, file_name, mime_type, size_bytes, height, width, duration_ms) VALUES
(
    (SELECT id FROM acme_org),
    (SELECT id FROM alice),
    'e2dfe414-a20c-41f5-b1a9-889e7604ff28',
    'video_1.mp4',
    'video/mp4',
    5120000,
    720,
    1280,
    60000
),
(
    (SELECT id FROM acme_org),
    (SELECT id FROM bob),
    '3bb17b52-8fc9-4686-acca-4874fadf79a3',
    'audio_1.mp3',
    'audio/mpeg',
    5120000,
    720,
    1280,
    60000
);
