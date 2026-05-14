ALTER TABLE groups
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'groups_status_check'
    ) THEN
        ALTER TABLE groups
            ADD CONSTRAINT groups_status_check
            CHECK (status IN ('active', 'locked'));
    END IF;
END $$;

ALTER TABLE group_members
    ALTER COLUMN role SET DEFAULT 'member';

CREATE INDEX IF NOT EXISTS idx_groups_creator
    ON groups(creator_id);

CREATE INDEX IF NOT EXISTS idx_groups_status
    ON groups(status);

CREATE INDEX IF NOT EXISTS idx_group_labs_lab_id
    ON group_labs(lab_id);

CREATE INDEX IF NOT EXISTS idx_group_starpaths_starpath_id
    ON group_starpaths(starpath_id);
