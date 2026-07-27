"""Cache-safe Python artifact source discovery.

@spec #2774
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import PurePath


__aw_artifact_id__ = "artifact:project-local-td-and-ec-gates/python-ec-cache-safe-discovery"
__aw_work_item__ = "2774"


IGNORED_RUNTIME_DIRECTORIES = frozenset(
    {
        "__pycache__",
        ".venv",
        "venv",
        ".pytest_cache",
        ".mypy_cache",
        ".ruff_cache",
        ".tox",
        "build",
        "dist",
        ".eggs",
    }
)


@dataclass(frozen=True)
class PythonSourceDiscoveryPolicy:
    """Select only author-owned Python sources for the canonical digest."""

    source_extension: str = ".py"
    ignored_directories: frozenset[str] = IGNORED_RUNTIME_DIRECTORIES

    def descend_into(self, directory_name: str) -> bool:
        """Reject standard runtime cache, virtualenv, and build products."""

        return directory_name not in self.ignored_directories

    def collect_file(self, relative_path: PurePath) -> bool:
        """Collect a Python file only when no ignored directory owns it."""

        return (
            relative_path.suffix == self.source_extension
            and all(
                self.descend_into(part)
                for part in relative_path.parts[:-1]
            )
        )


def cache_artifact_contract() -> tuple[PurePath, ...]:
    """Representative paths the implementation and black-box EC must ignore."""

    return (
        PurePath("src/__pycache__/contract.cpython-312.pyc"),
        PurePath("src/native-extension.so"),
        PurePath("src/build/generated.py"),
        PurePath("src/build/manifest.txt"),
        PurePath("src/build/opaque-cache"),
    )


def declared_contract_path_is_valid(path: PurePath) -> bool:
    """Declared EC sources stay stricter than ambient source-root discovery."""

    return (
        not path.is_absolute()
        and path.parts[:1] == ("src",)
        and path.suffix == ".py"
        and PythonSourceDiscoveryPolicy().collect_file(path)
    )
