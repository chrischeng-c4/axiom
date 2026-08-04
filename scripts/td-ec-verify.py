#!/usr/bin/env python3
"""Controller-owned TD/EC verification harness.

Reproduces the discipline of `aw ec verify --stage td` without depending on the
`aw` CLI: the EC must independently verify the full-typed Python TD across the
behavior and security dimensions, and must be *proved* discriminating rather
than merely observed green.

    python3 scripts/td-ec-verify.py libs/peer-tls
    python3 scripts/td-ec-verify.py libs/peer-tls --mutations scripts/td-ec-mutations/peer-tls.toml

Gates, in order. Any failure is fatal and exits 1.

    G1  td-unit-tests        tech-design/tests            unittest, all pass
    G2  ec-unit-tests        external-contracts/tests     unittest, all pass
    G3  case-inventory       pyproject.toml <-> src/      bijection, no orphans
    G4  runner-protocol      one JSON envelope, exit code matches status,
                             evidence written, per case
    G5  check-floor          len(checks) >= MINIMUM_CHECKS, 0 failing
    G5b declared-arity       MINIMUM_CHECKS == matrix entries == rows appended
    G6  answer-independence  every *_MATRIX expected value is a literal
    G7  self-report          no check whose "expected" is its own observation
    G8  distinct-checks      no two checks in a case share an observation
    G9  mutation-suite       every seeded TD defect is caught by >=1 case

G9 is the only gate that measures discriminating power. G1-G8 are necessary,
not sufficient: a suite can pass all eight and still be blind.
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

PROTOCOL = "aw.python-artifact.v1"


# --------------------------------------------------------------------------
# result plumbing
# --------------------------------------------------------------------------


@dataclass
class Gate:
    gate_id: str
    ok: bool
    detail: str
    lines: list[str] = field(default_factory=list)


class Report:
    def __init__(self) -> None:
        self.gates: list[Gate] = []

    def add(self, gate_id: str, ok: bool, detail: str, lines: list[str] | None = None) -> Gate:
        gate = Gate(gate_id, ok, detail, lines or [])
        self.gates.append(gate)
        mark = "PASS" if ok else "FAIL"
        print(f"[{mark}] {gate_id:<20} {detail}")
        for line in gate.lines:
            print(f"         {line}")
        return gate

    @property
    def ok(self) -> bool:
        return all(g.ok for g in self.gates)


# --------------------------------------------------------------------------
# project model
# --------------------------------------------------------------------------


@dataclass
class Project:
    root: Path
    td_dir: Path
    ec_dir: Path
    entrypoint: Path
    evidence_dir: Path
    cases: list[dict]

    @classmethod
    def load(cls, root: Path) -> "Project":
        ec_dir = root / "external-contracts"
        manifest = ec_dir / "pyproject.toml"
        if not manifest.is_file():
            raise SystemExit(f"no EC inventory at {manifest}")
        data = tomllib.loads(manifest.read_text())
        artifact = data["tool"]["aw"]["python-artifact"]
        if artifact["protocol"] != PROTOCOL:
            raise SystemExit(f"unsupported artifact protocol {artifact['protocol']!r}")
        return cls(
            root=root,
            td_dir=root / "tech-design",
            ec_dir=ec_dir,
            entrypoint=ec_dir / artifact["entrypoint"],
            evidence_dir=ec_dir / artifact["evidence_dir"],
            cases=data["tool"]["aw"]["python-ec"]["cases"],
        )


# --------------------------------------------------------------------------
# G1 / G2 — unit tests
# --------------------------------------------------------------------------


def run_unittests(report: Report, gate_id: str, tests_root: Path) -> None:
    if not tests_root.is_dir():
        report.add(gate_id, False, f"missing test tree {tests_root}")
        return
    # Each directory holding test modules is its own discovery root: the trees
    # carry no __init__.py, so a parent start-dir is not importable.
    dirs = sorted({p.parent for p in tests_root.rglob("test_*.py")})
    if not dirs:
        report.add(gate_id, False, f"no test modules under {tests_root}")
        return

    problems: list[str] = []
    ran = 0
    for directory in dirs:
        proc = subprocess.run(
            [sys.executable, "-m", "unittest", "discover", "-s", str(directory), "-t", str(directory)],
            capture_output=True,
            text=True,
            cwd=directory,
        )
        output = (proc.stderr or proc.stdout).strip()
        for line in output.splitlines():
            if line.startswith("Ran "):
                ran += int(line.split()[1])
        if proc.returncode != 0:
            problems.append(f"{directory.name}: {output.splitlines()[-1] if output else 'no output'}")

    report.add(gate_id, not problems, f"Ran {ran} tests in {len(dirs)} module dir(s)", problems)


# --------------------------------------------------------------------------
# G3 — inventory bijection
# --------------------------------------------------------------------------


def gate_inventory(report: Report, project: Project) -> None:
    declared = {c["test_path"] for c in project.cases}
    on_disk = {
        f"src/{p.name}"
        for p in (project.ec_dir / "src").glob("*.py")
        if p.name != project.entrypoint.name
    }
    missing = sorted(declared - on_disk)
    orphan = sorted(on_disk - declared)
    problems = [f"declared but absent: {p}" for p in missing]
    problems += [f"present but undeclared: {p}" for p in orphan]

    dims = {}
    for case in project.cases:
        dims.setdefault(case["capability_id"], set()).add(case["dimension"])
    for cap, seen in sorted(dims.items()):
        for required in ("behavior", "security"):
            if required not in seen:
                problems.append(f"capability {cap} has no {required} case")

    report.add(
        "case-inventory",
        not problems,
        f"{len(declared)} cases over {len(dims)} capabilities",
        problems,
    )


# --------------------------------------------------------------------------
# G4 / G5 — run every case through the real runner
# --------------------------------------------------------------------------


def purge_bytecode(root: Path) -> None:
    """Delete every cached .pyc under `root`.

    A .pyc records its source mtime in whole seconds. The mutation suite
    rewrites a TD module and restores it, usually within the same second and
    often without changing the byte count -- which leaves the mutant's cached
    bytecode valid for the restored source and silently inverts the answer.

    PYTHONDONTWRITEBYTECODE cannot prevent this: the protocol runs the case as
    `python -I`, and -I implies -E, so the interpreter ignores every PYTHON*
    variable we set. Purging before each invocation is the only lever we have.
    """
    for cache in root.rglob("__pycache__"):
        shutil.rmtree(cache, ignore_errors=True)


def run_case(project: Project, command: str) -> tuple[int, str, str]:
    purge_bytecode(project.td_dir / "src")
    purge_bytecode(project.ec_dir / "src")
    env = dict(
        os.environ,
        AW_PYTHON_ARTIFACT_PROTOCOL=PROTOCOL,
        AW_PYTHON_ARTIFACT_SOURCE_DIGEST="sha256:controller",
        AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST="sha256:controller",
        AW_PYTHON_ARTIFACT_EVIDENCE_DIR=str(project.evidence_dir.relative_to(project.ec_dir)),
    )
    proc = subprocess.run(
        [
            "uv", "run", "--frozen", "--offline", "python", "-I",
            str(project.entrypoint.relative_to(project.ec_dir)), command,
        ],
        capture_output=True,
        text=True,
        cwd=project.ec_dir,
        env=env,
    )
    return proc.returncode, proc.stdout, proc.stderr


def gate_execution(report: Report, project: Project) -> dict[str, dict]:
    if project.evidence_dir.is_dir():
        shutil.rmtree(project.evidence_dir)  # stale evidence has produced false readings

    protocol_problems: list[str] = []
    floor_problems: list[str] = []
    results: dict[str, dict] = {}
    total_checks = 0

    for case in project.cases:
        command = case["command"]
        code, out, err = run_case(project, command)

        try:
            envelope = json.loads(out)
        except json.JSONDecodeError:
            protocol_problems.append(f"{command}: stdout is not one JSON envelope ({err.strip()[:120]})")
            continue

        status = envelope.get("status")
        if status == "passed" and code != 0:
            protocol_problems.append(f"{command}: status=passed but exit={code}")
        if status == "failed" and code != 1:
            protocol_problems.append(f"{command}: status=failed but exit={code}")

        result: dict = {}
        for rel in case["evidence_paths"]:
            evidence = project.ec_dir / rel
            if not evidence.is_file():
                protocol_problems.append(f"{command}: evidence not written at {rel}")
                continue
            result = json.loads(evidence.read_text())

        checks = result.get("checks", [])
        floor = result.get("minimum_checks", 0)
        failing = [c["name"] for c in checks if not c.get("passed")]
        results[command] = result
        total_checks += len(checks)

        if len(checks) < floor:
            floor_problems.append(f"{command}: {len(checks)} checks < floor {floor}")
        if failing:
            floor_problems.append(f"{command}: failing {failing}")

    report.add(
        "runner-protocol",
        not protocol_problems,
        f"{len(project.cases)} cases through {project.entrypoint.name}",
        protocol_problems,
    )
    report.add(
        "check-floor",
        not floor_problems,
        f"{total_checks} checks, 0 failing" if not floor_problems else f"{total_checks} checks",
        floor_problems,
    )
    return results


# --------------------------------------------------------------------------
# G6 / G7 / G8 — static audits over the case source
# --------------------------------------------------------------------------


def _dict_entry(node: ast.Dict, key: str) -> ast.expr | None:
    for k, v in zip(node.keys, node.values):
        if isinstance(k, ast.Constant) and k.value == key:
            return v
    return None


def _is_literal(node: ast.expr) -> bool:
    if isinstance(node, ast.Constant):
        return True
    if isinstance(node, (ast.Tuple, ast.List, ast.Set)):
        return all(_is_literal(e) for e in node.elts)
    if isinstance(node, ast.Dict):
        # A `**other` spread carries a None key and can smuggle in a symbol.
        return all(k is not None and _is_literal(k) for k in node.keys) and all(
            _is_literal(v) for v in node.values
        )
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return _is_literal(node.operand)
    return False


def audit_case_source(path: Path) -> tuple[list[str], list[str], list[str], list[str]]:
    """Return (independence, self-report, duplicate, arity) problems for one case file."""
    tree = ast.parse(path.read_text())
    name = path.name
    independence: list[str] = []
    self_report: list[str] = []
    duplicate: list[str] = []
    arity: list[str] = []

    # G5b — MINIMUM_CHECKS, the matrix, and the appended rows must agree. A
    # report can claim a count; only the file can prove one.
    floor: int | None = None
    matrix_len: int | None = None
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        for target in node.targets:
            if not isinstance(target, ast.Name):
                continue
            if target.id == "MINIMUM_CHECKS" and isinstance(node.value, ast.Constant):
                floor = int(node.value.value)
            elif target.id.endswith("_MATRIX") and isinstance(node.value, (ast.Tuple, ast.List)):
                matrix_len = len(node.value.elts)
    appended = sum(
        1
        for n in ast.walk(tree)
        if isinstance(n, ast.Call)
        and isinstance(n.func, ast.Attribute)
        and n.func.attr == "append"
        and n.args
        and isinstance(n.args[0], ast.Dict)
    )
    if floor is None or matrix_len is None:
        arity.append(f"{name}: missing MINIMUM_CHECKS or *_MATRIX")
    elif not (floor == matrix_len == appended):
        arity.append(
            f"{name}: MINIMUM_CHECKS={floor}, matrix entries={matrix_len}, "
            f"rows appended={appended} — the three must agree"
        )

    # G6 — every *_MATRIX expected element must be a plain literal, never a
    # symbol read out of the design under test.
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        targets = [t.id for t in node.targets if isinstance(t, ast.Name)]
        if not any(t.endswith("_MATRIX") for t in targets):
            continue
        if not isinstance(node.value, (ast.Tuple, ast.List)):
            continue
        for index, entry in enumerate(node.value.elts):
            if not isinstance(entry, (ast.Tuple, ast.List)) or len(entry.elts) < 2:
                independence.append(f"{name}: matrix entry {index} is not a (name, expected) pair")
                continue
            expected = entry.elts[1]
            if not _is_literal(expected):
                independence.append(
                    f"{name}:{expected.lineno} entry {index} expected side is "
                    f"`{ast.unparse(expected)}` — not a literal"
                )

    # Resolve simple `obsN = <expr>` bindings so observations can be compared.
    # A name assigned more than once (the `try/except` accept-or-raise idiom)
    # is ambiguous and is left unresolved rather than guessed at.
    bindings: dict[str, str] = {}
    rebound: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
            if isinstance(target, ast.Name):
                if target.id in bindings:
                    rebound.add(target.id)
                bindings[target.id] = ast.unparse(node.value)
    for name in rebound:
        bindings.pop(name, None)

    seen_observations: dict[str, str] = {}
    for node in ast.walk(tree):
        if not (
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "append"
            and node.args
            and isinstance(node.args[0], ast.Dict)
        ):
            continue
        payload = node.args[0]
        expected = _dict_entry(payload, "expected")
        observed = _dict_entry(payload, "observed")
        check_name = _dict_entry(payload, "name")
        if expected is None or observed is None:
            continue

        # G7 — a row whose reported expectation is its own observation always
        # agrees with itself and tells the reader nothing.
        if ast.unparse(expected) == ast.unparse(observed):
            self_report.append(
                f"{name}:{payload.lineno} reports expected == observed "
                f"(`{ast.unparse(observed)}`)"
            )

        # G8 — two rows computing the same observation are one row plus padding.
        key = bindings.get(ast.unparse(observed))
        if key is None:
            continue  # unresolved binding — not enough information to call it a repeat
        label = ast.unparse(check_name) if check_name is not None else f"line {payload.lineno}"
        if key in seen_observations:
            duplicate.append(
                f"{name}:{payload.lineno} {label} repeats the observation of "
                f"{seen_observations[key]} (`{key}`)"
            )
        else:
            seen_observations[key] = label

    return independence, self_report, duplicate, arity


def gate_static_audits(report: Report, project: Project) -> None:
    independence: list[str] = []
    self_report: list[str] = []
    duplicate: list[str] = []
    arity: list[str] = []
    for case in project.cases:
        a, b, c, d = audit_case_source(project.ec_dir / case["test_path"])
        independence += a
        self_report += b
        duplicate += c
        arity += d

    report.add("declared-arity", not arity, "MINIMUM_CHECKS == matrix == rows appended", arity)
    report.add("answer-independence", not independence, "expected values are contract literals", independence)
    report.add("self-report", not self_report, "no row reports its own observation", self_report)
    report.add("distinct-checks", not duplicate, "every row observes something different", duplicate)


# --------------------------------------------------------------------------
# G9 — mutation suite
# --------------------------------------------------------------------------


def gate_mutations(report: Report, project: Project, mutations_path: Path | None) -> None:
    if mutations_path is None:
        report.add("mutation-suite", False, "no mutation file supplied — discriminating power unproved")
        return
    spec = tomllib.loads(mutations_path.read_text())
    mutations = spec.get("mutation", [])
    if not mutations:
        report.add("mutation-suite", False, f"{mutations_path} declares no mutations")
        return

    src_root = project.td_dir / "src"
    blind: list[str] = []
    caught = 0

    # run_case purges before every invocation; this only clears bytecode left
    # behind by whatever ran before this harness did.
    purge_bytecode(src_root)

    # A case that already fails on unmutated source exits nonzero for every
    # seed, so it "catches" all of them without observing anything. Measuring
    # discriminating power against a red baseline reports a number that cannot
    # be wrong, which is worse than reporting none.
    red_at_rest = [
        case["command"]
        for case in project.cases
        if run_case(project, case["command"])[0] != 0
    ]
    if red_at_rest:
        report.add(
            "mutation-suite",
            False,
            "baseline is not green — discriminating power is unmeasurable",
            [
                f"{command}: fails on unmutated source, so it catches every seed vacuously"
                for command in red_at_rest
            ],
        )
        return

    for mutation in mutations:
        target = src_root / mutation["file"]
        original = target.read_text()
        hits = original.count(mutation["old"])
        if hits != 1:
            blind.append(
                f"{mutation['id']}: anchor occurs {hits} times in {mutation['file']} "
                "— a defect that is not uniquely placed proves nothing"
            )
            continue
        target.write_text(original.replace(mutation["old"], mutation["new"], 1))
        try:
            detectors = [
                case["command"]
                for case in project.cases
                if run_case(project, case["command"])[0] != 0
            ]
        finally:
            target.write_text(original)
        if detectors:
            caught += 1
        else:
            blind.append(f"{mutation['id']}: BLIND — {mutation['intent']}")

    report.add(
        "mutation-suite",
        not blind,
        f"{caught}/{len(mutations)} seeded defects caught",
        blind,
    )


# --------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", type=Path, help="project root, e.g. libs/peer-tls")
    parser.add_argument("--mutations", type=Path, default=None)
    parser.add_argument("--skip-mutations", action="store_true")
    args = parser.parse_args()

    project = Project.load(args.project.resolve())
    print(f"verifying {args.project} — TD {project.td_dir.name}, EC {project.ec_dir.name}\n")

    report = Report()
    run_unittests(report, "td-unit-tests", project.td_dir / "tests")
    run_unittests(report, "ec-unit-tests", project.ec_dir / "tests")
    gate_inventory(report, project)
    gate_execution(report, project)
    gate_static_audits(report, project)
    if not args.skip_mutations:
        gate_mutations(report, project, args.mutations)

    failed = [g.gate_id for g in report.gates if not g.ok]
    print()
    if failed:
        print(f"REJECTED — {len(failed)} gate(s) failed: {', '.join(failed)}")
        return 1
    print(f"ACCEPTED — {len(report.gates)}/{len(report.gates)} gates green")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
