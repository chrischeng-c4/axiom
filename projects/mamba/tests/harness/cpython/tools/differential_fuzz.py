#!/usr/bin/env python3.12
"""Deterministic differential fuzz harness for issue #1125.

This v1 harness mutates a small, deterministic seed set drawn from the fixture
tree, runs each mutant under CPython 3.12 and mamba, and classifies:

* `MATCH`    same exit code and stdout
* `DIVERGE`  stdout or exit code differs
* `CRASH`    mamba timed out, signaled, or emitted panic/assertion output

It is intentionally stdlib-only and fast enough for local smoke runs or nightly
bounded batches.
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import shutil
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import harness_lib  # noqa: E402


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"

DEFAULT_TIMEOUT = 10
DEFAULT_MAX_CASES = 6
DEFAULT_MUTATIONS_PER_SEED = 2
DEFAULT_FACTORS = (2, 4, 8)
TEXT_TAIL = 160

DEFAULT_SEEDS = (
    "_regression/core/compiler_resilience/debug_assertions_smoke_small.py",
    "_regression/core/compiler_resilience/deep_nesting.py",
    "_regression/core/compiler_resilience/oversized_int.py",
    "security/std-libs/json/recursion_bomb_raises_not_crash.py",
    "security/std-libs/xml_etree_elementtree/malformed_untrusted_input_raises_cleanly.py",
)


@dataclass(frozen=True)
class IntSite:
    index: int
    value: int
    lineno: int
    col: int


@dataclass(frozen=True)
class MutationCase:
    seed_rel: str
    mutation_id: str
    operator: str
    site: IntSite
    factor: int
    source: str


class IntegerCollector(ast.NodeVisitor):
    def __init__(self) -> None:
        self.sites: list[IntSite] = []

    def visit_Constant(self, node: ast.Constant) -> None:  # noqa: N802
        if type(node.value) is int and 0 < node.value <= 100_000:
            self.sites.append(
                IntSite(
                    index=len(self.sites),
                    value=node.value,
                    lineno=getattr(node, "lineno", 0),
                    col=getattr(node, "col_offset", 0),
                )
            )
        self.generic_visit(node)


class IntegerAmplifier(ast.NodeTransformer):
    def __init__(self, target_index: int, factor: int, cap: int) -> None:
        self.target_index = target_index
        self.factor = factor
        self.cap = cap
        self.current = 0

    def visit_Constant(self, node: ast.Constant) -> ast.AST:  # noqa: N802
        if type(node.value) is int and 0 < node.value <= 100_000:
            idx = self.current
            self.current += 1
            if idx == self.target_index:
                new_value = min(node.value * self.factor, self.cap)
                return ast.copy_location(ast.Constant(value=new_value), node)
        return self.generic_visit(node)


def repo_rel(path: Path) -> str:
    return path.relative_to(FIXTURES_DIR).as_posix()


def fixture_path(rel: str) -> Path:
    path = FIXTURES_DIR / rel
    if not path.is_file():
        raise SystemExit(f"fixture not found: {rel}")
    return path


def default_mamba_bin() -> str:
    if override := os.environ.get("MAMBA_BIN", "").strip():
        return override
    for candidate in (
        MAMBA_DIR.parents[1] / "target" / "debug" / "mamba",
        MAMBA_DIR.parents[1] / "target" / "release" / "mamba",
    ):
        if candidate.is_file():
            return str(candidate)
    return shutil.which("mamba") or "mamba"


def default_python_bin() -> str:
    if override := os.environ.get("MAMBA_ORACLE_PYTHON", "").strip():
        return override
    oracle_env = FIXTURES_DIR / ".cache" / "oracle-env" / "bin" / "python3"
    if oracle_env.is_file():
        return str(oracle_env)
    return shutil.which("python3.12") or shutil.which("python3") or "python3"


def collect_sites(source: str) -> list[IntSite]:
    tree = ast.parse(source)
    collector = IntegerCollector()
    collector.visit(tree)
    return collector.sites


def build_mutants(
    seed_rel: str,
    *,
    factor_cycle: tuple[int, ...],
    mutations_per_seed: int,
    int_cap: int,
) -> list[MutationCase]:
    source = fixture_path(seed_rel).read_text(encoding="utf-8")
    sites = collect_sites(source)
    if not sites:
        return []
    out: list[MutationCase] = []
    for site in sites[:mutations_per_seed]:
        factor = factor_cycle[site.index % len(factor_cycle)]
        mutated = IntegerAmplifier(site.index, factor, int_cap).visit(ast.parse(source))
        ast.fix_missing_locations(mutated)
        mutant_source = ast.unparse(mutated) + "\n"
        out.append(
            MutationCase(
                seed_rel=seed_rel,
                mutation_id=f"{Path(seed_rel).stem}-int{site.index}-x{factor}",
                operator="amplify_integer_literal",
                site=site,
                factor=factor,
                source=mutant_source,
            )
        )
    return out


def output_tail(value: str | bytes) -> str:
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    value = value.strip()
    if len(value) <= TEXT_TAIL:
        return value
    return value[-TEXT_TAIL:]


def looks_like_panic(stdout: str, stderr: str) -> bool:
    haystack = f"{stdout}\n{stderr}".lower()
    return (
        "panicked at" in haystack
        or "assertion failed:" in haystack
        or "thread 'main' panicked" in haystack
    )


def run_case(argv: list[str], timeout: int) -> dict[str, Any]:
    start = time.monotonic()
    rc, stdout, stderr = harness_lib.run_fixture(argv, timeout, text=False)
    stdout_s = output_tail(stdout)
    stderr_s = output_tail(stderr)
    elapsed = round(time.monotonic() - start, 4)
    status = "ok"
    if rc is None:
        status = "timeout"
    elif rc < 0:
        status = f"signal:{-rc}"
    elif looks_like_panic(stdout_s, stderr_s):
        status = "panic_output"
    elif rc != 0:
        status = f"rc:{rc}"
    return {
        "command": argv,
        "returncode": rc,
        "status": status,
        "stdout_tail": stdout_s,
        "stderr_tail": stderr_s,
        "elapsed_s": elapsed,
    }


def classify(oracle: dict[str, Any], mamba: dict[str, Any]) -> tuple[str, str]:
    if mamba["status"] in {"timeout", "panic_output"} or (
        isinstance(mamba["status"], str) and mamba["status"].startswith("signal:")
    ):
        return "CRASH", f"mamba {mamba['status']}"
    if oracle["returncode"] == mamba["returncode"] and oracle["stdout_tail"] == mamba["stdout_tail"]:
        return "MATCH", "same exit code and stdout tail"
    if oracle["returncode"] != mamba["returncode"]:
        return "DIVERGE", f"exit mismatch oracle={oracle['returncode']} mamba={mamba['returncode']}"
    return "DIVERGE", "stdout mismatch"


def materialize_mutants(cases: list[MutationCase], out_dir: Path) -> list[tuple[MutationCase, Path]]:
    materialized: list[tuple[MutationCase, Path]] = []
    out_dir.mkdir(parents=True, exist_ok=True)
    for case in cases:
        path = out_dir / f"{case.mutation_id}.py"
        header = (
            f"# generated by differential_fuzz.py\n"
            f"# seed = {case.seed_rel}\n"
            f"# operator = {case.operator}\n"
            f"# factor = {case.factor}\n"
            f"# original_integer = {case.site.value}\n\n"
        )
        path.write_text(header + case.source, encoding="utf-8")
        materialized.append((case, path))
    return materialized


def list_seeds() -> list[dict[str, Any]]:
    payload: list[dict[str, Any]] = []
    for rel in DEFAULT_SEEDS:
        source = fixture_path(rel).read_text(encoding="utf-8")
        payload.append(
            {
                "seed": rel,
                "integer_sites": [site.__dict__ for site in collect_sites(source)],
            }
        )
    return payload


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Deterministic differential fuzz harness over CPython fixture seeds.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list-seeds", help="show the default deterministic seed set")
    list_parser.add_argument("--json", action="store_true", help="emit JSON")

    run_parser = subparsers.add_parser("run", help="materialize mutants and diff mamba vs CPython")
    run_parser.add_argument(
        "--seed",
        dest="seeds",
        action="append",
        default=None,
        help="relative fixture path to use as a seed; repeatable",
    )
    run_parser.add_argument("--mamba-bin", default=default_mamba_bin(), help="mamba binary to run")
    run_parser.add_argument("--python", default=default_python_bin(), help="CPython oracle binary")
    run_parser.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT, help="per-process timeout in seconds")
    run_parser.add_argument(
        "--mutations-per-seed",
        type=int,
        default=DEFAULT_MUTATIONS_PER_SEED,
        help="number of integer-literal mutations to derive from each seed",
    )
    run_parser.add_argument(
        "--max-cases",
        type=int,
        default=DEFAULT_MAX_CASES,
        help="global cap on materialized mutant cases",
    )
    run_parser.add_argument(
        "--int-cap",
        type=int,
        default=250_000,
        help="maximum amplified integer literal value",
    )
    run_parser.add_argument("--json", action="store_true", help="emit JSON")
    run_parser.add_argument("--keep-dir", help="keep generated mutants in this directory")
    return parser.parse_args(argv)


def print_human(result: dict[str, Any]) -> None:
    print(
        "summary:"
        f" MATCH={result['counts'].get('MATCH', 0)}"
        f" DIVERGE={result['counts'].get('DIVERGE', 0)}"
        f" CRASH={result['counts'].get('CRASH', 0)}"
    )
    for case in result["cases"]:
        print(
            f"{case['status']:>7}  {case['seed']} -> {case['generated_file']}"
            f"  {case['reason']}"
        )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.command == "list-seeds":
        payload = list_seeds()
        if args.json:
            print(json.dumps(payload, indent=2, sort_keys=True))
        else:
            for item in payload:
                print(f"{item['seed']}:")
                for site in item["integer_sites"]:
                    print(
                        f"  - int[{site['index']}] value={site['value']}"
                        f" at {site['lineno']}:{site['col']}"
                    )
        return 0

    seeds = args.seeds or list(DEFAULT_SEEDS)
    mutants: list[MutationCase] = []
    for seed in seeds:
        mutants.extend(
            build_mutants(
                seed,
                factor_cycle=DEFAULT_FACTORS,
                mutations_per_seed=args.mutations_per_seed,
                int_cap=args.int_cap,
            )
        )
    mutants = mutants[: args.max_cases]
    if not mutants:
        raise SystemExit("no mutants were generated from the selected seeds")

    temp_context = None
    if args.keep_dir:
        out_dir = Path(args.keep_dir)
    else:
        temp_context = tempfile.TemporaryDirectory(prefix="mamba_diff_fuzz_")
        out_dir = Path(temp_context.name)

    materialized = materialize_mutants(mutants, out_dir)
    counts: dict[str, int] = {}
    cases: list[dict[str, Any]] = []
    for mutant, generated in materialized:
        oracle = run_case([args.python, str(generated)], args.timeout)
        mamba = run_case([args.mamba_bin, "run", str(generated)], args.timeout)
        status, reason = classify(oracle, mamba)
        counts[status] = counts.get(status, 0) + 1
        cases.append(
            {
                "seed": mutant.seed_rel,
                "mutation_id": mutant.mutation_id,
                "operator": mutant.operator,
                "generated_file": str(generated),
                "site": mutant.site.__dict__,
                "factor": mutant.factor,
                "status": status,
                "reason": reason,
                "oracle": oracle,
                "mamba": mamba,
            }
        )

    result = {
        "counts": counts,
        "cases": cases,
        "mamba_bin": args.mamba_bin,
        "python": args.python,
        "seed_count": len(seeds),
        "mutant_count": len(materialized),
        "out_dir": str(out_dir),
    }
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print_human(result)

    if temp_context is not None:
        temp_context.cleanup()
    return 0 if counts.get("CRASH", 0) == 0 and counts.get("DIVERGE", 0) == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
