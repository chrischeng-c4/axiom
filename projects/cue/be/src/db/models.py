"""SQLAlchemy ORM models for Cue control-plane storage."""

from datetime import datetime
from uuid import UUID

from sqlalchemy import CheckConstraint, DateTime, ForeignKey, Index, Integer, Text, func, text
from sqlalchemy.dialects.postgresql import JSONB, UUID as PgUUID
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship


class Base(DeclarativeBase):
    """Base class for Cue ORM models."""


class Project(Base):
    __tablename__ = "cue_projects"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    name: Mapped[str] = mapped_column(Text, nullable=False)
    owner: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(Text, nullable=False)
    next_action: Mapped[str] = mapped_column(Text, nullable=False)
    summary: Mapped[str] = mapped_column(Text, nullable=False, default="")
    active_session_id: Mapped[str | None] = mapped_column(
        Text,
        ForeignKey("cue_sessions.id", ondelete="SET NULL", deferrable=True, initially="DEFERRED"),
        nullable=True,
    )
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    sessions: Mapped[list["Session"]] = relationship(
        "Session",
        foreign_keys="Session.project_id",
        cascade="all, delete-orphan",
    )
    workitems: Mapped[list["WorkItem"]] = relationship("WorkItem", cascade="all, delete-orphan")
    artifacts: Mapped[list["Artifact"]] = relationship("Artifact", cascade="all, delete-orphan")


class Session(Base):
    __tablename__ = "cue_sessions"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    project_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_projects.id", ondelete="CASCADE"), nullable=False)
    title: Mapped[str] = mapped_column(Text, nullable=False)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    messages: Mapped[list["Message"]] = relationship("Message", cascade="all, delete-orphan")

    __table_args__ = (Index("idx_cue_sessions_project", "project_id"),)


class Message(Base):
    __tablename__ = "cue_messages"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    project_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_projects.id", ondelete="CASCADE"), nullable=False)
    session_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_sessions.id", ondelete="CASCADE"), nullable=False)
    speaker: Mapped[str] = mapped_column(Text, nullable=False)
    body: Mapped[str] = mapped_column(Text, nullable=False)
    action: Mapped[str | None] = mapped_column(Text, nullable=True)
    classification: Mapped[str | None] = mapped_column(Text, nullable=True)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    __table_args__ = (
        CheckConstraint("speaker IN ('cue', 'owner', 'agent', 'system')", name="ck_cue_messages_speaker"),
        Index("idx_cue_messages_session_created", "session_id", "created_at"),
    )


class WorkItem(Base):
    __tablename__ = "cue_workitems"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    project_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_projects.id", ondelete="CASCADE"), nullable=False)
    title: Mapped[str] = mapped_column(Text, nullable=False)
    route: Mapped[str] = mapped_column(Text, nullable=False)
    target: Mapped[str] = mapped_column(Text, nullable=False)
    state: Mapped[str] = mapped_column(Text, nullable=False)
    progress: Mapped[int] = mapped_column(Integer, nullable=False, default=0)
    next_action: Mapped[str] = mapped_column(Text, nullable=False)
    blockers: Mapped[list] = mapped_column(JSONB, nullable=False, default=list)
    workflow_plan: Mapped[list] = mapped_column(JSONB, nullable=False, default=list)
    qc_status: Mapped[str] = mapped_column(Text, nullable=False, default="pending")
    qc_checks: Mapped[list] = mapped_column(JSONB, nullable=False, default=list)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    artifacts: Mapped[list["Artifact"]] = relationship("Artifact")

    __table_args__ = (
        CheckConstraint("progress >= 0 AND progress <= 100", name="ck_cue_workitems_progress"),
        Index("idx_cue_workitems_project_state", "project_id", "state"),
        Index("idx_cue_workitems_project_target", "project_id", "target"),
    )


class Artifact(Base):
    __tablename__ = "cue_artifacts"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    project_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_projects.id", ondelete="CASCADE"), nullable=False)
    workitem_id: Mapped[str | None] = mapped_column(Text, ForeignKey("cue_workitems.id", ondelete="SET NULL"), nullable=True)
    label: Mapped[str] = mapped_column(Text, nullable=False)
    kind: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(Text, nullable=False)
    summary: Mapped[str | None] = mapped_column(Text, nullable=True)
    entrypoints: Mapped[list] = mapped_column(JSONB, nullable=False, default=list)
    qc_status: Mapped[str] = mapped_column(Text, nullable=False, default="pending")
    qc_checks: Mapped[list] = mapped_column(JSONB, nullable=False, default=list)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    versions: Mapped[list["ArtifactVersion"]] = relationship("ArtifactVersion", cascade="all, delete-orphan")

    __table_args__ = (
        Index("idx_cue_artifacts_project_kind", "project_id", "kind"),
        Index("idx_cue_artifacts_workitem", "workitem_id"),
    )


class ArtifactVersion(Base):
    __tablename__ = "cue_artifact_versions"

    id: Mapped[str] = mapped_column(Text, primary_key=True)
    artifact_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_artifacts.id", ondelete="CASCADE"), nullable=False)
    version: Mapped[int] = mapped_column(Integer, nullable=False)
    status: Mapped[str] = mapped_column(Text, nullable=False)
    content: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    __table_args__ = (
        CheckConstraint("version > 0", name="ck_cue_artifact_versions_version"),
        Index("idx_cue_artifact_versions_artifact", "artifact_id", "version"),
    )


class QcRun(Base):
    __tablename__ = "cue_qc_runs"

    id: Mapped[UUID] = mapped_column(PgUUID(as_uuid=True), primary_key=True, server_default=text("gen_random_uuid()"))
    project_id: Mapped[str] = mapped_column(Text, ForeignKey("cue_projects.id", ondelete="CASCADE"), nullable=False)
    workitem_id: Mapped[str | None] = mapped_column(Text, ForeignKey("cue_workitems.id", ondelete="SET NULL"), nullable=True)
    artifact_id: Mapped[str | None] = mapped_column(Text, ForeignKey("cue_artifacts.id", ondelete="SET NULL"), nullable=True)
    target_type: Mapped[str] = mapped_column(Text, nullable=False)
    target_id: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(Text, nullable=False)
    summary: Mapped[str | None] = mapped_column(Text, nullable=True)
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    checks: Mapped[list["QcCheck"]] = relationship("QcCheck", cascade="all, delete-orphan")

    __table_args__ = (Index("idx_cue_qc_runs_target", "target_type", "target_id", "created_at"),)


class QcCheck(Base):
    __tablename__ = "cue_qc_checks"

    id: Mapped[UUID] = mapped_column(PgUUID(as_uuid=True), primary_key=True, server_default=text("gen_random_uuid()"))
    run_id: Mapped[UUID] = mapped_column(PgUUID(as_uuid=True), ForeignKey("cue_qc_runs.id", ondelete="CASCADE"), nullable=False)
    check_id: Mapped[str] = mapped_column(Text, nullable=False)
    label: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(Text, nullable=False)
    summary: Mapped[str] = mapped_column(Text, nullable=False, default="")
    meta: Mapped[dict] = mapped_column("metadata", JSONB, nullable=False, default=dict)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    __table_args__ = (Index("idx_cue_qc_checks_run", "run_id"),)
