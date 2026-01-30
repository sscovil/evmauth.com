-- These triggers enforce business logic for orgs and people in the auth schema.
-- They handle personal workspace creation, ownership transfer restrictions,
-- synchronization of ownership roles, and validation of membership constraints.

-- After INSERT on auth.people, create a personal workspace
CREATE OR REPLACE FUNCTION auth.tfn_people_create_personal_workspace()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO auth.orgs (owner_id, visibility, display_name)
    VALUES (NEW.id, 'personal', 'Personal Workspace');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER ait_people_create_personal_workspace
    AFTER INSERT ON auth.people
    FOR EACH ROW
EXECUTE FUNCTION auth.tfn_people_create_personal_workspace();


-- After INSERT or UPDATE of owner_id on auth.orgs, sync orgs_people roles
CREATE OR REPLACE FUNCTION auth.tfn_orgs_sync_owner_role()
RETURNS TRIGGER AS $$
BEGIN
    -- On UPDATE, demote the previous owner to admin (if still a member)
    IF TG_OP = 'UPDATE' THEN
        UPDATE auth.orgs_people
        SET role = 'admin'
        WHERE org_id = NEW.id
          AND member_id = OLD.owner_id
          AND role = 'owner';
    END IF;

    -- Ensure new owner has role='owner' in orgs_people
    INSERT INTO auth.orgs_people (org_id, member_id, role)
    VALUES (NEW.id, NEW.owner_id, 'owner')
    ON CONFLICT (org_id, member_id)
    DO UPDATE SET role = 'owner';

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aiut_orgs_sync_owner_role
    AFTER INSERT OR UPDATE OF owner_id ON auth.orgs
    FOR EACH ROW
EXECUTE FUNCTION auth.tfn_orgs_sync_owner_role();


-- Before INSERT or UPDATE on auth.orgs_people, validate membership constraints
CREATE OR REPLACE FUNCTION auth.tfn_orgs_people_validate_membership()
RETURNS TRIGGER AS $$
DECLARE
    org_owner_id UUID;
    org_visibility TEXT;
BEGIN
    -- Get org info in a single query
    SELECT o.owner_id, o.visibility INTO org_owner_id, org_visibility
    FROM auth.orgs o
    WHERE o.id = NEW.org_id;

    -- Validate role='owner' matches orgs.owner_id
    IF NEW.role = 'owner' AND NEW.member_id != org_owner_id THEN
        RAISE EXCEPTION 'Only the org owner (owner_id) can have role=''owner'' in orgs_people';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER biut_orgs_people_validate_membership
    BEFORE INSERT OR UPDATE ON auth.orgs_people
    FOR EACH ROW
EXECUTE FUNCTION auth.tfn_orgs_people_validate_membership();


-- Before UPDATE of visibility or DELETE on auth.orgs, retain one personal workspace
CREATE OR REPLACE FUNCTION auth.tfn_orgs_retain_at_least_one_personal_workspace()
RETURNS TRIGGER AS $$
DECLARE
    personal_workspace_count INT;
BEGIN
    -- Check if the org being deleted/updated is a personal workspace
    IF OLD.visibility = 'personal' THEN
        -- Count how many personal workspaces the owner has
        SELECT COUNT(*) INTO personal_workspace_count
        FROM auth.orgs
        WHERE owner_id = OLD.owner_id AND visibility = 'personal';

        -- If this is the only personal workspace, prevent deletion or visibility change
        IF personal_workspace_count <= 1 THEN
            RAISE EXCEPTION 'Each person must have at least one personal workspace';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER budt_orgs_retain_at_least_one_personal_workspace
    BEFORE DELETE OR UPDATE OF visibility ON auth.orgs
    FOR EACH ROW
EXECUTE FUNCTION auth.tfn_orgs_retain_at_least_one_personal_workspace();
