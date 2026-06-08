"""Cue control-plane SQL schema.

Plain SQL is intentional here: it is easy to run through SQLAlchemy, asyncpg,
psql, or Alembic. This file is the reference contract for the first PG-backed
Cue slice.
"""

CONTROL_PLANE_UP_SQL = [
    "CREATE EXTENSION IF NOT EXISTS pgcrypto",
    """
    CREATE TABLE IF NOT EXISTS cue_projects (
        id                  TEXT PRIMARY KEY,
        name                TEXT NOT NULL,
        owner               TEXT NOT NULL,
        status              TEXT NOT NULL,
        next_action         TEXT NOT NULL,
        summary             TEXT NOT NULL DEFAULT '',
        active_session_id   TEXT,
        metadata            JSONB NOT NULL DEFAULT '{}',
        created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_sessions (
        id          TEXT PRIMARY KEY,
        project_id  TEXT NOT NULL REFERENCES cue_projects(id) ON DELETE CASCADE,
        title       TEXT NOT NULL,
        metadata    JSONB NOT NULL DEFAULT '{}',
        created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    DO $$
    BEGIN
        IF NOT EXISTS (
            SELECT 1 FROM pg_constraint
            WHERE conname = 'cue_projects_active_session_fk'
        ) THEN
            ALTER TABLE cue_projects
            ADD CONSTRAINT cue_projects_active_session_fk
            FOREIGN KEY (active_session_id)
            REFERENCES cue_sessions(id)
            ON DELETE SET NULL
            DEFERRABLE INITIALLY DEFERRED;
        END IF;
    END $$;
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_messages (
        id              TEXT PRIMARY KEY,
        project_id      TEXT NOT NULL REFERENCES cue_projects(id) ON DELETE CASCADE,
        session_id      TEXT NOT NULL REFERENCES cue_sessions(id) ON DELETE CASCADE,
        speaker         TEXT NOT NULL CHECK (speaker IN ('cue', 'owner', 'agent', 'system')),
        body            TEXT NOT NULL,
        action          TEXT,
        classification  TEXT,
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_workitems (
        id              TEXT PRIMARY KEY,
        project_id      TEXT NOT NULL REFERENCES cue_projects(id) ON DELETE CASCADE,
        title           TEXT NOT NULL,
        route           TEXT NOT NULL,
        target          TEXT NOT NULL,
        state           TEXT NOT NULL,
        progress        INTEGER NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
        next_action     TEXT NOT NULL,
        blockers        JSONB NOT NULL DEFAULT '[]',
        workflow_plan   JSONB NOT NULL DEFAULT '[]',
        qc_status       TEXT NOT NULL DEFAULT 'pending',
        qc_checks       JSONB NOT NULL DEFAULT '[]',
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_artifacts (
        id              TEXT PRIMARY KEY,
        project_id      TEXT NOT NULL REFERENCES cue_projects(id) ON DELETE CASCADE,
        workitem_id     TEXT REFERENCES cue_workitems(id) ON DELETE SET NULL,
        label           TEXT NOT NULL,
        kind            TEXT NOT NULL,
        status          TEXT NOT NULL,
        summary         TEXT,
        entrypoints     JSONB NOT NULL DEFAULT '[]',
        qc_status       TEXT NOT NULL DEFAULT 'pending',
        qc_checks       JSONB NOT NULL DEFAULT '[]',
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_artifact_versions (
        id              TEXT PRIMARY KEY,
        artifact_id     TEXT NOT NULL REFERENCES cue_artifacts(id) ON DELETE CASCADE,
        version         INTEGER NOT NULL CHECK (version > 0),
        status          TEXT NOT NULL,
        content         JSONB NOT NULL DEFAULT '{}',
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
        UNIQUE (artifact_id, version)
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_qc_runs (
        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        project_id      TEXT NOT NULL REFERENCES cue_projects(id) ON DELETE CASCADE,
        workitem_id     TEXT REFERENCES cue_workitems(id) ON DELETE SET NULL,
        artifact_id     TEXT REFERENCES cue_artifacts(id) ON DELETE SET NULL,
        target_type     TEXT NOT NULL,
        target_id       TEXT NOT NULL,
        status          TEXT NOT NULL,
        summary         TEXT,
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    """
    CREATE TABLE IF NOT EXISTS cue_qc_checks (
        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        run_id          UUID NOT NULL REFERENCES cue_qc_runs(id) ON DELETE CASCADE,
        check_id        TEXT NOT NULL,
        label           TEXT NOT NULL,
        status          TEXT NOT NULL,
        summary         TEXT NOT NULL DEFAULT '',
        metadata        JSONB NOT NULL DEFAULT '{}',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    )
    """,
    "CREATE INDEX IF NOT EXISTS idx_cue_sessions_project ON cue_sessions(project_id)",
    "CREATE INDEX IF NOT EXISTS idx_cue_messages_session_created ON cue_messages(session_id, created_at)",
    "CREATE INDEX IF NOT EXISTS idx_cue_workitems_project_state ON cue_workitems(project_id, state)",
    "CREATE INDEX IF NOT EXISTS idx_cue_workitems_project_target ON cue_workitems(project_id, target)",
    "CREATE INDEX IF NOT EXISTS idx_cue_artifacts_project_kind ON cue_artifacts(project_id, kind)",
    "CREATE INDEX IF NOT EXISTS idx_cue_artifacts_workitem ON cue_artifacts(workitem_id)",
    "CREATE INDEX IF NOT EXISTS idx_cue_artifact_versions_artifact ON cue_artifact_versions(artifact_id, version DESC)",
    "CREATE INDEX IF NOT EXISTS idx_cue_qc_runs_target ON cue_qc_runs(target_type, target_id, created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_cue_qc_checks_run ON cue_qc_checks(run_id)",
]

CONTROL_PLANE_DOWN_SQL = [
    "DROP TABLE IF EXISTS cue_qc_checks CASCADE",
    "DROP TABLE IF EXISTS cue_qc_runs CASCADE",
    "DROP TABLE IF EXISTS cue_artifact_versions CASCADE",
    "DROP TABLE IF EXISTS cue_artifacts CASCADE",
    "DROP TABLE IF EXISTS cue_workitems CASCADE",
    "DROP TABLE IF EXISTS cue_messages CASCADE",
    "ALTER TABLE IF EXISTS cue_projects DROP CONSTRAINT IF EXISTS cue_projects_active_session_fk",
    "DROP TABLE IF EXISTS cue_sessions CASCADE",
    "DROP TABLE IF EXISTS cue_projects CASCADE",
]
