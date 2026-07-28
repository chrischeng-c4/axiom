"""Executable design for Guard's library API surface."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-library-api"


@dataclass(frozen=True)
class LibraryModule:
    name: str
    responsibility: str


def guard_library_modules() -> tuple[LibraryModule, ...]:
    return (
        LibraryModule("scan", "Compass-backed policy evaluation"),
        LibraryModule("report", "guard.report/1 projection and persistence"),
        LibraryModule("evidence", "external security evidence normalization"),
    )
