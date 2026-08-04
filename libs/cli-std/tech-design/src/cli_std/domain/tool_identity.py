from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ToolInfo:
    project: str
    repo: str
    target: str
    version: str
    git_sha: str
    built_at: str

    def issue_label(self) -> str:
        return f"app:{self.project}"

    def tag_prefix(self) -> str:
        return f"{self.project}@"

    def asset_name(self) -> str:
        return f"{self.project}-{self.target}.tar.gz"

    def inner_binary_path(self) -> str:
        return f"{self.project}-{self.target}/{self.project}"
