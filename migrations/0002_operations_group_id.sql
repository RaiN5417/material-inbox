-- Move operations didn't record which group they were filing into, so crash
-- reconciliation (spec section 39) couldn't fully restore a file's state
-- after an interrupted move — it could tell the move probably finished, but
-- not which group to attribute it to. See docs/architecture.md.

-- ON DELETE SET NULL (not NO ACTION, unlike files.group_id): this is an
-- audit trail, so a deleted group shouldn't be permanently un-deletable just
-- because it has history. files.group_id staying NO ACTION is what actually
-- protects live data — a group with files still in it can't be deleted.
ALTER TABLE operations ADD COLUMN group_id TEXT REFERENCES groups (id) ON DELETE SET NULL;
