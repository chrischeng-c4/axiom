from __future__ import annotations

import json
from dataclasses import dataclass

from openapi_codegen.infrastructure.options import GeneratedOutput

MANIFEST_FILE = ".openapi-codegen.json"


@dataclass(frozen=True)
class GenerationManifest:
    schema_version: int
    generator: str
    compiler: str
    target: str
    language: str
    minimum_version: str
    language_standard: str
    module_system: str | None
    module_resolution: str | None
    strict: bool | None
    transport: str | None
    runtime_dependencies: tuple[str, ...]


def manifest_of(output: GeneratedOutput) -> GenerationManifest | None:
    if output.requirements is None:
        return None
    r = output.requirements
    return GenerationManifest(
        schema_version=1,
        generator="openapi-codegen",
        compiler=r.compiler,
        target=r.target,
        language=r.language.id,
        minimum_version=r.minimum_version,
        language_standard=r.language_standard,
        module_system=r.module_system,
        module_resolution=r.module_resolution,
        strict=r.strict,
        transport=r.transport,
        runtime_dependencies=tuple(r.runtime_dependencies),
    )


def serialize_manifest(m: GenerationManifest) -> str:
    d = {
        "schema_version": m.schema_version,
        "generator": m.generator,
        "compiler": m.compiler,
        "target": m.target,
        "language": m.language,
        "minimum_version": m.minimum_version,
        "language_standard": m.language_standard,
        "module_system": m.module_system,
        "module_resolution": m.module_resolution,
        "strict": m.strict,
        "transport": m.transport,
        "runtime_dependencies": list(m.runtime_dependencies),
    }
    return json.dumps(d, indent=2) + "\n"


def manifest_fields(m: GenerationManifest) -> tuple[tuple[str, object], ...]:
    return (
        ("schema_version", m.schema_version),
        ("generator", m.generator),
        ("compiler", m.compiler),
        ("target", m.target),
        ("language", m.language),
        ("minimum_version", m.minimum_version),
        ("language_standard", m.language_standard),
        ("module_system", m.module_system),
        ("module_resolution", m.module_resolution),
        ("strict", m.strict),
        ("transport", m.transport),
        ("runtime_dependencies", m.runtime_dependencies),
    )
