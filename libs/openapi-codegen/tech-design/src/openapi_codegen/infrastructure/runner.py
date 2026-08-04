from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Protocol

from openapi_codegen.domain.output_path import OutputPathEscape, joined_output_path
from openapi_codegen.infrastructure.manifest import MANIFEST_FILE, manifest_of
from openapi_codegen.infrastructure.options import GenOptions, GeneratedOutput

# Sentinel constant representing manifest contents in pure infrastructure layer
MANIFEST_SENTINEL = "<manifest>"


class FileSystem(Protocol):
    def read_text(self, path: str) -> str | None: ...
    def write_text(self, path: str, contents: str) -> None: ...


@dataclass(frozen=True)
class RunResult:
    exit_code: int
    stdout: tuple[str, ...]
    stderr: tuple[str, ...]


def write_plan(
    output: GeneratedOutput, out_dir: str
) -> tuple[tuple[str, str], ...] | OutputPathEscape:
    # Deliberate strengthening over Rust: evaluate all paths before returning
    # any write pairs, ensuring complete fail-closed atomic validation.
    pairs: list[tuple[str, str]] = []
    for file in output.files:
        joined = joined_output_path(out_dir, file.rel_path)
        if isinstance(joined, OutputPathEscape):
            return joined
        pairs.append((joined, file.contents))
    m = manifest_of(output)
    if m is not None:
        pairs.append((out_dir + "/" + MANIFEST_FILE, MANIFEST_SENTINEL))
    return tuple(pairs)


def run(
    opts: GenOptions,
    fs: FileSystem,
    generate: Callable[[str, GenOptions], GeneratedOutput | str],
) -> RunResult:
    spec = fs.read_text(opts.spec_path)
    if spec is None:
        return RunResult(
            2, (), (f"openapi-codegen: cannot read {opts.spec_path}",)
        )

    result = generate(spec, opts)
    if isinstance(result, str):
        return RunResult(1, (), (f"openapi-codegen: {result}",))

    plan = write_plan(result, opts.out_dir)
    if isinstance(plan, OutputPathEscape):
        return RunResult(
            1,
            (),
            (
                f"openapi-codegen: cannot write generated output to "
                f"{opts.out_dir}: {plan.message()}",
            ),
        )

    for path, contents in plan:
        fs.write_text(path, contents)

    stdout: list[str] = [
        f"generated {joined_output_path(opts.out_dir, f.rel_path)}"
        for f in result.files
    ]
    if result.target is not None:
        stdout.append(f"generated {opts.out_dir}/{MANIFEST_FILE}")

    return RunResult(0, tuple(stdout), ())
