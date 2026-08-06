from __future__ import annotations

from dataclasses import dataclass

from openapi_codegen.domain.lang import Lang


@dataclass(frozen=True)
class UnknownTargetProfile:
    value: str

    def message(self) -> str:
        from openapi_codegen.domain.target import KNOWN_TARGET_IDS

        joined = ", ".join(KNOWN_TARGET_IDS)
        return f"unknown target profile '{self.value}'; expected one of: {joined}"


@dataclass(frozen=True)
class PolicyLanguageMismatch:
    key: str
    expected: Lang
    got: str

    def message(self) -> str:
        return f"{self.key} must select {self.expected.id}, got {self.got}"


@dataclass(frozen=True)
class TargetLanguageMismatch:
    profile_id: str
    profile_lang: Lang
    requested: Lang

    def message(self) -> str:
        return (
            f"target profile {self.profile_id} is for {self.profile_lang.id}, "
            f"not requested language {self.requested.id}"
        )


@dataclass(frozen=True)
class MissingPolicyKey:
    key: str

    def message(self) -> str:
        return f"target policy is missing required key {self.key}"


@dataclass(frozen=True)
class OutputPathEscape:
    rel_path: str
    reason: str

    def message(self) -> str:
        return (
            f"generated file path must stay under output directory: "
            f"'{self.rel_path}' ({self.reason})"
        )
