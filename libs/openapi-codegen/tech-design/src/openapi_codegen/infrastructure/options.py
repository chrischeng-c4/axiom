from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import (
    TargetProfile,
    TargetRequirements,
    profile_requirements,
)


class HttpClient(Enum):
    FETCH = "fetch"
    AXIOS = "axios"


def default_http_client() -> HttpClient:
    return HttpClient.FETCH


@dataclass(frozen=True)
class GenOptions:
    lang: Lang
    target: TargetProfile | None
    spec_path: str
    out_dir: str
    client_name: str
    http_client: HttpClient
    emit_types: bool
    emit_client: bool
    emit_hooks: bool


@dataclass(frozen=True)
class GeneratedFile:
    rel_path: str
    contents: str


@dataclass(frozen=True)
class GeneratedOutput:
    files: tuple[GeneratedFile, ...]
    target: TargetProfile | None
    requirements: TargetRequirements | None


def legacy(
    files: tuple[GeneratedFile, ...] | list[GeneratedFile]
) -> GeneratedOutput:
    return GeneratedOutput(
        files=tuple(files),
        target=None,
        requirements=None,
    )


def for_target(
    files: tuple[GeneratedFile, ...] | list[GeneratedFile],
    target: TargetProfile,
) -> GeneratedOutput:
    return GeneratedOutput(
        files=tuple(files),
        target=target,
        requirements=profile_requirements(target),
    )


def default_output() -> GeneratedOutput:
    return legacy(())
