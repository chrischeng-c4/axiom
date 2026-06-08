"""Alembic-compatible migration for Cue control-plane tables."""

try:
    from db.schema import CONTROL_PLANE_DOWN_SQL, CONTROL_PLANE_UP_SQL
except ImportError:
    from src.db.schema import CONTROL_PLANE_DOWN_SQL, CONTROL_PLANE_UP_SQL

revision = "001_control_plane"
down_revision = None
branch_labels = None
depends_on = None


def upgrade(op_obj=None) -> None:
    """Apply the schema through Alembic's op object."""
    if op_obj is None:
        from alembic import op as op_obj
    for statement in CONTROL_PLANE_UP_SQL:
        op_obj.execute(statement)


def downgrade(op_obj=None) -> None:
    """Drop the schema through Alembic's op object."""
    if op_obj is None:
        from alembic import op as op_obj
    for statement in CONTROL_PLANE_DOWN_SQL:
        op_obj.execute(statement)
