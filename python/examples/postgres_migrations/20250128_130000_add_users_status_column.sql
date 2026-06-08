-- Migration: 20250128_130000_add_users_status_column
-- Description: Add status column to users table

-- UP
ALTER TABLE users
    ADD COLUMN status VARCHAR(50) DEFAULT 'active' NOT NULL;

CREATE INDEX idx_users_status ON users(status);

-- Add check constraint
ALTER TABLE users
    ADD CONSTRAINT check_users_status
    CHECK (status IN ('active', 'inactive', 'suspended', 'deleted'));

-- DOWN
ALTER TABLE users DROP CONSTRAINT IF EXISTS check_users_status;
DROP INDEX IF EXISTS idx_users_status;
ALTER TABLE users DROP COLUMN IF EXISTS status;
