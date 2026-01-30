-- Create two users: Alice and Bob
-- Note: Default "Private Workspace" orgs are created automatically via trigger
INSERT INTO auth.people (display_name, auth_provider_name, auth_provider_ref, primary_email) VALUES
('Alice', 'turnkey', '123456', 'alice@example.com'),
('Bob', 'privy', '234567', 'bob@example.com'),
('Carol', 'google', '345678', 'carol@example.com');

-- Create "Acme Corp" owned by Alice, and private group owned by Bob
-- Note: Alice is automatically added as owner in orgs_people via trigger
INSERT INTO auth.orgs (display_name, owner_id, visibility) VALUES
('Acme Corp', (SELECT id FROM auth.people WHERE primary_email = 'alice@example.com'), 'public'),
('Bobby''s Hobbies', (SELECT id FROM auth.people WHERE primary_email = 'bob@example.com'), 'private');

-- Add Bob as admin to Acme Corp, and Carol as a member of Bob's private group
INSERT INTO auth.orgs_people (org_id, member_id, role) VALUES
(
    (SELECT id FROM auth.orgs WHERE display_name = 'Acme Corp'),
    (SELECT id FROM auth.people WHERE primary_email = 'bob@example.com'),
    'admin'
),
(
    (SELECT id FROM auth.orgs WHERE display_name = 'Bobby''s Hobbies'),
    (SELECT id FROM auth.people WHERE primary_email = 'carol@example.com'),
    'member'
);
