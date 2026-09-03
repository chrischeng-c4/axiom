#!/usr/bin/env python3
"""The single phase for maintenance delivery work items.

Maintenance is not a weakened behavior flow.  It has its own evidence shape:
the controller runs every command declared by the work item, then ``record``
binds the exact command, exit code, and sha256 of its captured output.  This
script never runs a command copied from an issue body.

The verbs, in order, are:

  start <iid>     validate the staged delivery, require a clean tree, and pin
                  HEAD plus the work-item bytes in a baseline record
  record <iid>    record one controller-run gate without retaining its output
  verify <iid>    check the diff, its type-specific boundary, and all records
  commit <iid>    re-run verify and land exactly the measured paths

``refactor`` records the same behavior gates before and after the change.
``test``, ``docs``, and ``chore`` record their declared gates after the change.
No maintenance type manufactures a red observation.
"""
from __future__ import annotations

import argparse
import difflib
import fnmatch
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any

# The type registry is the only lifecycle module imported at startup.  The
# change facade is in flight independently of this file, so its schema hook is
# loaded inside ``validate_staged`` and nowhere else.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from wi_types import AW_CLI, MAINTENANCE_TYPES, flow_for  # noqa: E402


GIT = ("git", "-c", "core.fsmonitor=false")
PHASE = "maint"
RECORD_DIR = Path(".aw") / "maint"
STAGED_DIR = Path(".aw") / "workitems" / "deliveries"
SCHEMA_VERSION = 1

PRODUCT_DOC_NAMES = frozenset({"README.md", "STATUS.md", "ROADMAP.md"})
CONFIG_EXTENSIONS = frozenset({".toml", ".yaml", ".yml", ".ini", ".cfg", ".conf"})
CHORE_NAMES = frozenset({
    "Cargo.toml", "Cargo.lock", "build.rs", "Makefile", "Justfile",
    "Dockerfile", ".dockerignore", "package.json", "package-lock.json",
    "pnpm-lock.yaml", "yarn.lock", "pyproject.toml", "uv.lock",
    "requirements.txt", "go.mod", "go.sum", "rust-toolchain",
    "rust-toolchain.toml", "deny.toml",
})
CHORE_DIRS = frozenset({
    ".cargo", ".github", "build", "ci", "config", "configs", "scripts",
    "tools", "tooling",
})


class MaintError(RuntimeError):
    """One maintenance invariant refused the requested verb."""


class Check:
    def __init__(self) -> None:
        self.rows: list[tuple[str, str, str]] = []

    def add(self, status: str, name: str, detail: str = "") -> None:
        self.rows.append((status, name, detail))

    @property
    def failed(self) -> bool:
        return any(status == "FAIL" for status, _name, _detail in self.rows)

    def report(self) -> None:
        for status, name, detail in self.rows:
            print(f"  {status:8s} {name}")
            for line in detail.splitlines():
                print(f"           {line}")


def git(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([*GIT, *args], cwd=repo, capture_output=True, text=True)


def repo_root(start: Path | None = None) -> Path:
    here = (start or Path.cwd()).resolve()
    proc = subprocess.run(
        [*GIT, "rev-parse", "--show-toplevel"], cwd=here,
        capture_output=True, text=True,
    )
    if proc.returncode != 0 or not proc.stdout.strip():
        raise MaintError(
            f"not inside a git checkout: {proc.stderr.strip() or here}"
        )
    root = Path(proc.stdout.strip()).resolve()
    if not (root / "aw.toml").is_file():
        raise MaintError(f"{root} has no root aw.toml")
    return root


def project_root(repo: Path, project: str) -> Path:
    candidates = [repo / family / project for family in ("apps", "libs")]
    found = [path for path in candidates if path.is_dir()]
    if len(found) != 1:
        rendered = ", ".join(str(path.relative_to(repo)) for path in found) or "<none>"
        raise MaintError(
            f"--project {project!r} must resolve to exactly one app or lib; found {rendered}"
        )
    return found[0]


def head_sha(repo: Path) -> str:
    proc = git(repo, "rev-parse", "HEAD")
    if proc.returncode != 0 or not proc.stdout.strip():
        raise MaintError(proc.stderr.strip() or "cannot resolve HEAD")
    return proc.stdout.strip()


def dirty_paths(repo: Path) -> list[str]:
    """Return both sides of every rename and every untracked file."""
    proc = subprocess.run(
        [*GIT, "status", "--porcelain=v1", "-z", "-uall"], cwd=repo,
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        raise MaintError(proc.stderr.strip() or "git status failed")
    records = proc.stdout.split("\0")
    paths: list[str] = []
    index = 0
    while index < len(records):
        record = records[index]
        index += 1
        if not record:
            continue
        if len(record) < 4:
            raise MaintError(f"cannot parse git status entry: {record!r}")
        status, path = record[:2], record[3:]
        paths.append(path)
        if "R" in status or "C" in status:
            if index >= len(records) or not records[index]:
                raise MaintError(f"git reported {status!r} without its source path")
            paths.append(records[index])
            index += 1
    return sorted(set(paths))


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_digest(value: Any) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True,
    ).encode("utf-8")
    return sha256_bytes(encoded)


def record_path(repo: Path, iid: int) -> Path:
    return repo / RECORD_DIR / f"{iid}.json"


def _save_record(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    scratch = path.with_suffix(f".json.{os.getpid()}.tmp")
    scratch.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8",
    )
    scratch.replace(path)


def _load_json(path: Path, subject: str) -> dict[str, Any]:
    if not path.is_file():
        raise MaintError(f"no {subject} at {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise MaintError(f"{subject} at {path} is not readable JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise MaintError(f"{subject} at {path} is not a JSON object")
    return value


def _section(text: str, heading: str, level: int) -> str:
    lines = text.splitlines()
    wanted = heading.strip().lower()
    start: int | None = None
    prefix = "#" * level + " "
    shallower = tuple("#" * depth + " " for depth in range(1, level + 1))
    for index, line in enumerate(lines):
        stripped = line.strip()
        if start is None:
            if stripped.lower() == wanted:
                start = index + 1
            continue
        if stripped.startswith(shallower):
            return "\n".join(lines[start:index])
    return "" if start is None else "\n".join(lines[start:])


def declared_gates(body: str) -> list[str]:
    acceptance = _section(body, "## Acceptance", 2)
    commands: list[str] = []
    for line in acceptance.splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [
            cell.strip().replace("\\|", "|")
            for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))
        ]
        if len(cells) < 5 or cells[0].lower() in {"#", "---"}:
            continue
        if all(cell and set(cell) <= {"-", ":"} for cell in cells):
            continue
        matches = re.findall(r"`([^`\n]+)`", cells[1])
        if len(matches) != 1 or cells[1] != f"`{matches[0]}`":
            raise MaintError(
                "each Acceptance command cell must be exactly one backticked command; "
                f"found {cells[1]!r}"
            )
        command = matches[0]
        if not command or command != command.strip():
            raise MaintError(f"Acceptance carries a non-canonical command {command!r}")
        if command in commands:
            raise MaintError(f"Acceptance repeats the exact command `{command}`")
        commands.append(command)
    if not commands:
        raise MaintError("the staged body declares no exact Acceptance command")
    return commands


_PATH_TOKEN = re.compile(
    r"(?:\.?[A-Za-z0-9_*-]+/)+(?:[A-Za-z0-9_.{}*?\[\]-]+)|"
    r"(?:README|STATUS|ROADMAP|Cargo|package|pyproject|uv|requirements|go|"
    r"rust-toolchain|deny|Makefile|Justfile|Dockerfile)[A-Za-z0-9_.{}*?\[\]-]*"
)


def _normalise_declared(token: str, owner: str) -> str:
    value = token.strip().lstrip("`'\"(,;").rstrip("`'\"(),.;")
    value = re.sub(r":\d+$", "", value)
    while value.startswith("./"):
        value = value[2:]
    if not value or value.startswith("/"):
        raise MaintError(f"change point path must be repository-relative: {token!r}")
    parts = PurePosixPath(value).parts
    if ".." in parts:
        raise MaintError(f"change point path escapes the repository: {token!r}")
    repo_roots = {
        "apps", "libs", ".github", ".cargo", "scripts", "tools", "tooling",
        "ci", "config", "configs", "build", "acceptance",
    }
    root_names = CHORE_NAMES | {"CONTRIBUTING.md", "LICENSE", "LICENSE.md"}
    if parts and (parts[0] in repo_roots or value in root_names):
        return value.rstrip("/")
    return f"{owner}/{value}".rstrip("/")


def declared_paths(body: str, owner: str) -> list[str]:
    how = _section(body, "## How", 2)
    points = _section(how, "### Change points", 3)
    found: list[str] = []
    for line in points.splitlines():
        stripped = line.lstrip()
        if not stripped.startswith(("- ", "* ", "+ ")):
            continue
        item = stripped[2:].strip()
        quoted = re.findall(r"`([^`\n]+)`", item)
        candidates = quoted or _PATH_TOKEN.findall(item)
        for token in candidates:
            # A backticked command is not a write target.  The GHAN validator
            # already requires each change-point item to carry a path; this
            # check keeps a second code-like span from silently widening it.
            if "/" not in token and token not in CHORE_NAMES | PRODUCT_DOC_NAMES:
                continue
            normal = _normalise_declared(token, owner)
            if normal not in found:
                found.append(normal)
    if not found:
        raise MaintError("the staged body declares no usable Change points path")
    return found


def _load_change_schema() -> Any:
    """Load only the facade hook that owns GHAN body validation."""
    key = "_aw_maint_change_schema"
    if key in sys.modules:
        return sys.modules[key]
    try:
        path = Path(__file__).resolve().parent / "change.py"
        spec = importlib.util.spec_from_file_location(key, path)
        if spec is None or spec.loader is None:
            raise ImportError(f"cannot create an import spec for {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[key] = module
        spec.loader.exec_module(module)
    except (ImportError, SystemExit) as exc:
        sys.modules.pop(key, None)
        raise MaintError(f"cannot load the change body validator: {exc}") from exc
    if not hasattr(module, "CHANGE_TYPES") or not hasattr(module, "validate_body"):
        raise MaintError(
            "change.py exposes neither CHANGE_TYPES nor validate_body; maintenance "
            "cannot guess whether the staged body is valid"
        )
    return module


def validate_staged(repo: Path, owner: Path, iid: int) -> dict[str, Any]:
    body_path = repo / STAGED_DIR / f"{iid}.md"
    receipt_path = repo / STAGED_DIR / f"{iid}.json"
    if not body_path.is_file():
        raise MaintError(
            f"no staged delivery body at {body_path}\n"
            f"run: {AW_CLI} change fetch {iid}"
        )
    body = body_path.read_text(encoding="utf-8")
    receipt = _load_json(receipt_path, "staged delivery receipt")
    if receipt.get("iid") != iid:
        raise MaintError(
            f"staged receipt names #{receipt.get('iid')}, not requested #{iid}"
        )
    kind = str(receipt.get("type") or "")
    if kind not in MAINTENANCE_TYPES:
        rendered = ", ".join(MAINTENANCE_TYPES)
        raise MaintError(
            f"#{iid} has type:{kind or '<none>'}; maint accepts only {rendered}"
        )
    try:
        flow = flow_for(kind)
    except ValueError as exc:
        raise MaintError(str(exc)) from exc
    if flow != "maintenance" or receipt.get("flow") != flow:
        raise MaintError(
            f"staged receipt does not bind type:{kind} to the maintenance flow"
        )
    if str(receipt.get("state") or "").upper() != "OPEN":
        raise MaintError(f"#{iid} is not open in the staged receipt")
    labels = [str(label) for label in receipt.get("labels") or []]
    type_labels = sorted(label for label in labels if label.startswith("type:"))
    if type_labels != [f"type:{kind}"]:
        rendered = ", ".join(type_labels) or "<none>"
        raise MaintError(
            f"staged receipt needs exactly type:{kind}; found {rendered}"
        )
    owner_rel = owner.relative_to(repo).as_posix()
    family, project = owner_rel.split("/", 1)
    expected_owner = f"{'app' if family == 'apps' else 'lib'}:{project}"
    owner_labels = sorted(
        label for label in labels if label.startswith(("app:", "lib:"))
    )
    if owner_labels != [expected_owner]:
        rendered = ", ".join(owner_labels) or "<none>"
        raise MaintError(
            f"staged receipt for {owner_rel} needs exactly {expected_owner}; found {rendered}"
        )
    phase_labels = sorted(label for label in labels if label.startswith("phase:"))
    if phase_labels != ["phase:created"]:
        rendered = ", ".join(phase_labels) or "<none>"
        raise MaintError(
            f"#{iid} must be in phase:created when maintenance starts; found {rendered}"
        )
    body_sha = sha256_bytes(body.encode("utf-8"))
    if receipt.get("body_sha256") != body_sha:
        raise MaintError(
            "staged body bytes do not match the tracker receipt; run "
            f"`{AW_CLI} change fetch {iid}` again"
        )

    change = _load_change_schema()
    wi_type = change.CHANGE_TYPES.get(kind)
    if wi_type is None:
        raise MaintError(f"change.py has no GHAN schema binding for type:{kind}")
    try:
        errors = [
            error for error in change.validate_body(body, wi_type)
            if not str(error).startswith("note:")
        ]
    except TypeError as exc:
        raise MaintError(
            "change.py validate_body interface is incompatible; maintenance "
            "refuses instead of validating with a local substitute"
        ) from exc
    if errors:
        raise MaintError(
            f"#{iid} is not a valid type:{kind} delivery body:\n"
            + "\n".join(str(error) for error in errors[:8])
        )

    return {
        "iid": iid,
        "type": kind,
        "flow": flow,
        "body": body,
        "body_sha256": body_sha,
        "receipt_sha256": sha256_bytes(receipt_path.read_bytes()),
        "gates": declared_gates(body),
        "declared_paths": declared_paths(body, owner_rel),
        "owner": owner_rel,
    }


def load_baseline(repo: Path, iid: int) -> dict[str, Any]:
    value = _load_json(record_path(repo, iid), "maintenance baseline record")
    if value.get("schema") != SCHEMA_VERSION:
        raise MaintError(
            f"maintenance baseline schema is {value.get('schema')!r}, expected {SCHEMA_VERSION}"
        )
    return value


def _matches_declared(path: str, patterns: list[str]) -> bool:
    for pattern in patterns:
        if any(char in pattern for char in "*?["):
            if fnmatch.fnmatchcase(path, pattern):
                return True
            if pattern.endswith("/**") and path == pattern[:-3].rstrip("/"):
                return True
        if path == pattern or path.startswith(pattern.rstrip("/") + "/"):
            return True
    return False


def change_digest(repo: Path, baseline: dict[str, Any], paths: list[str]) -> str:
    value: dict[str, Any] = {
        "base": baseline["base"],
        "body_sha256": baseline["body_sha256"],
        "paths": [],
    }
    for rel in sorted(paths):
        path = repo / rel
        value["paths"].append({
            "path": rel,
            "sha256": sha256_bytes(path.read_bytes()) if path.is_file() else "(absent)",
        })
    return canonical_digest(value)


def _is_under(path: str, prefix: str) -> bool:
    return path == prefix or path.startswith(prefix.rstrip("/") + "/")


def _is_test_file(path: str, owner: str) -> bool:
    if not _is_under(path, owner):
        return False
    rel = path[len(owner):].lstrip("/")
    parts = PurePosixPath(rel).parts
    name = PurePosixPath(rel).name.lower()
    if "e2e" in parts or "tests" in parts or "test" in parts:
        return True
    if name == "tests.rs" or name.startswith("test_"):
        return True
    return any(name.endswith(suffix) for suffix in (
        "_test.py", "_test.rs", ".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx",
        ".test.js", ".spec.js",
    ))


def _is_product_doc(path: str, owner: str) -> bool:
    if not _is_under(path, owner):
        return False
    rel = path[len(owner):].lstrip("/")
    pure = PurePosixPath(rel)
    return pure.name in PRODUCT_DOC_NAMES or (pure.parts and pure.parts[0] == "docs")


def _is_product_src(path: str, owner: str) -> bool:
    return _is_under(path, f"{owner}/src") and not _is_test_file(path, owner)


def _is_chore_path(path: str) -> bool:
    pure = PurePosixPath(path)
    if pure.name in CHORE_NAMES or pure.name.startswith("Dockerfile."):
        return True
    if pure.suffix in CONFIG_EXTENSIONS:
        # A source-tree config-shaped fixture is still product source.  The
        # caller applies the explicit src refusal before reaching this helper.
        return True
    return any(part in CHORE_DIRS for part in pure.parts)


def _read_head_file(repo: Path, rel: str) -> list[str]:
    proc = git(repo, "show", f"HEAD:{rel}")
    if proc.returncode != 0:
        return []
    return proc.stdout.splitlines()


def _read_tree_file(repo: Path, rel: str) -> list[str]:
    path = repo / rel
    if not path.is_file():
        return []
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError as exc:
        raise MaintError(f"{rel} is not UTF-8 text") from exc


def _changed_lines(repo: Path, rel: str) -> tuple[list[tuple[int, str]], list[tuple[int, str]], list[str], list[str]]:
    old, new = _read_head_file(repo, rel), _read_tree_file(repo, rel)
    before: list[tuple[int, str]] = []
    after: list[tuple[int, str]] = []
    matcher = difflib.SequenceMatcher(a=old, b=new, autojunk=False)
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            continue
        before.extend((index + 1, old[index]) for index in range(i1, i2))
        after.extend((index + 1, new[index]) for index in range(j1, j2))
    return before, after, old, new


def _rust_code_lines(lines: list[str]) -> list[str]:
    """Rust with comments and string contents removed, preserving line count.

    A quoted ``#[cfg(test)]`` or a brace inside fixture prose must not widen a
    test section.  This small lexer recognizes the multiline forms that can do
    that.  It deliberately leaves character literals and lifetimes alone:
    neither can span a line, and treating an apostrophe as a quote would erase
    ordinary lifetimes such as ``'a``.
    """
    out: list[str] = []
    block = False
    raw_end: str | None = None
    for line in lines:
        code: list[str] = []
        index = 0
        while index < len(line):
            if raw_end is not None:
                end = line.find(raw_end, index)
                if end < 0:
                    index = len(line)
                    continue
                index = end + len(raw_end)
                raw_end = None
                continue
            if block:
                end = line.find("*/", index)
                if end < 0:
                    index = len(line)
                    continue
                index = end + 2
                block = False
                continue
            if line.startswith("//", index):
                break
            if line.startswith("/*", index):
                block = True
                index += 2
                continue
            raw = re.match(r'r(#+)?"', line[index:])
            if raw:
                hashes = raw.group(1) or ""
                delimiter = '"' + hashes
                index += len(raw.group(0))
                end = line.find(delimiter, index)
                if end < 0:
                    raw_end = delimiter
                    index = len(line)
                else:
                    index = end + len(delimiter)
                continue
            if line[index] == '"':
                index += 1
                while index < len(line):
                    if line[index] == "\\":
                        index += 2
                    elif index < len(line) and line[index] == '"':
                        index += 1
                        break
                    else:
                        index += 1
                continue
            code.append(line[index])
            index += 1
        out.append("".join(code))
    return out


def _rust_test_mask(lines: list[str]) -> set[int]:
    """Conservatively identify complete items guarded by ``#[cfg(test)]``."""
    marked: set[int] = set()
    depth = 0
    pending = False
    active_parent: int | None = None
    code_lines = _rust_code_lines(lines)
    for number, (line, code) in enumerate(zip(lines, code_lines), start=1):
        stripped = code.strip()
        before_depth = depth
        if active_parent is not None:
            marked.add(number)
        if re.match(r"^#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", stripped):
            marked.add(number)
            pending = True
        elif pending:
            marked.add(number)
            if stripped and not stripped.startswith(("//", "/*", "*")):
                if "{" in code:
                    active_parent = before_depth
                    pending = False
                elif ";" in code:
                    pending = False
        depth += code.count("{") - code.count("}")
        if active_parent is not None and depth <= active_parent and "}" in code:
            active_parent = None
    return marked


def _test_only_change(repo: Path, rel: str, owner: str) -> tuple[bool, str]:
    if _is_test_file(rel, owner):
        return True, "dedicated test path"
    if not rel.endswith(".rs"):
        return False, "not a dedicated test path or a Rust #[cfg(test)] section"
    before, after, old, new = _changed_lines(repo, rel)
    old_mask, new_mask = _rust_test_mask(old), _rust_test_mask(new)
    bad_old = [number for number, line in before if line.strip() and number not in old_mask]
    bad_new = [number for number, line in after if line.strip() and number not in new_mask]
    if bad_old or bad_new:
        detail = []
        if bad_old:
            detail.append("HEAD lines " + ", ".join(map(str, bad_old[:8])))
        if bad_new:
            detail.append("tree lines " + ", ".join(map(str, bad_new[:8])))
        return False, "changed outside #[cfg(test)]: " + "; ".join(detail)
    return True, "all changed Rust lines are inside #[cfg(test)]"


def _comment_mask(lines: list[str], suffix: str) -> set[int]:
    """Lines whose non-whitespace bytes are entirely comment syntax.

    Multiline strings are tracked so quoted prose that starts with ``//`` or
    ``#`` cannot pass as a comment.  A block-comment delimiter sharing a line
    with executable text does not make that line documentation-only.
    """
    marked: set[int] = set()
    in_block = False
    block_end = ""
    raw_end: str | None = None
    triple_end: str | None = None
    hash_comments = suffix in {".py", ".sh", ".toml", ".yaml", ".yml"}
    dash_comments = suffix in {".sql"}
    for number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if raw_end is not None:
            if raw_end in line:
                raw_end = None
            continue
        if triple_end is not None:
            if triple_end in line:
                triple_end = None
            continue
        if not stripped:
            if in_block:
                marked.add(number)
            continue
        if in_block:
            end = stripped.find(block_end)
            if end < 0:
                marked.add(number)
                continue
            if not stripped[end + len(block_end):].strip():
                marked.add(number)
            in_block = False
            block_end = ""
            continue

        if stripped.startswith("//"):
            marked.add(number)
            continue
        if hash_comments and stripped.startswith("#"):
            marked.add(number)
            continue
        if dash_comments and stripped.startswith("--"):
            marked.add(number)
            continue

        pure_block = False
        for opening, closing in (("/*", "*/"), ("<!--", "-->")):
            if not stripped.startswith(opening):
                continue
            end = stripped.find(closing, len(opening))
            suffix_text = "" if end < 0 else stripped[end + len(closing):].strip()
            if not suffix_text:
                marked.add(number)
            if end < 0:
                in_block = True
                block_end = closing
            pure_block = True
            break
        if pure_block:
            continue

        # Track strings only on code lines.  A comment may quote ``r#\"`` or
        # triple quotes as prose; that must not turn following comments into
        # string contents.
        raw = re.search(r'r(#+)?"', line)
        if raw:
            delimiter = '"' + (raw.group(1) or "")
            if delimiter not in line[raw.end():]:
                raw_end = delimiter
        if suffix == ".py":
            for delimiter in ('"""', "'''"):
                first = line.find(delimiter)
                if first >= 0 and line.find(delimiter, first + 3) < 0:
                    triple_end = delimiter
                    break
    return marked


def _comment_only_change(repo: Path, rel: str) -> tuple[bool, str]:
    before, after, old, new = _changed_lines(repo, rel)
    suffix = PurePosixPath(rel).suffix.lower()
    old_mask, new_mask = _comment_mask(old, suffix), _comment_mask(new, suffix)
    bad_old = [number for number, line in before if line.strip() and number not in old_mask]
    bad_new = [number for number, line in after if line.strip() and number not in new_mask]
    if bad_old or bad_new:
        detail = []
        if bad_old:
            detail.append("HEAD lines " + ", ".join(map(str, bad_old[:8])))
        if bad_new:
            detail.append("tree lines " + ", ".join(map(str, bad_new[:8])))
        return False, "changed executable text: " + "; ".join(detail)
    return True, "all changed nonblank lines are comments"


def type_scope_errors(repo: Path, baseline: dict[str, Any], paths: list[str]) -> list[str]:
    kind = baseline["type"]
    owner = baseline["owner"]
    errors: list[str] = []
    outside_declared = [
        path for path in paths
        if not _matches_declared(path, list(baseline["declared_paths"]))
    ]
    if outside_declared:
        errors.append(
            "changed outside GHAN Change points:\n"
            + "\n".join(f"  {path}" for path in outside_declared)
        )
        return errors

    if kind == "refactor":
        outside = [path for path in paths if not _is_under(path, owner)]
        if outside:
            errors.append(
                "refactor changed outside its owning project:\n"
                + "\n".join(f"  {path}" for path in outside)
            )
        if not any(_is_product_src(path, owner) for path in paths):
            errors.append("refactor changed no product source under src/")
        return errors

    if kind == "test":
        for path in paths:
            if not _is_under(path, owner):
                errors.append(f"test changed outside its owning project: {path}")
                continue
            allowed, why = _test_only_change(repo, path, owner)
            if not allowed:
                errors.append(f"test changed product code in {path}: {why}")
        return errors

    if kind == "docs":
        for path in paths:
            if not _is_under(path, owner):
                errors.append(f"docs changed outside its owning project: {path}")
                continue
            if _is_product_doc(path, owner):
                continue
            allowed, why = _comment_only_change(repo, path)
            if not allowed:
                errors.append(f"docs changed non-document code in {path}: {why}")
        return errors

    if kind == "chore":
        for path in paths:
            if _is_under(path, f"{owner}/src"):
                errors.append(f"chore must not change product src: {path}")
            elif not _is_chore_path(path):
                errors.append(
                    f"chore path is not build/config/dependency/tooling: {path}"
                )
        return errors

    errors.append(f"no maintenance scope rule exists for type:{kind}")
    return errors


def _record_identity_matches(baseline: dict[str, Any], staged: dict[str, Any], project: str) -> None:
    expected = {
        "iid": staged["iid"],
        "project": project,
        "owner": staged["owner"],
        "type": staged["type"],
        "flow": staged["flow"],
        "body_sha256": staged["body_sha256"],
        "receipt_sha256": staged["receipt_sha256"],
        "gates": staged["gates"],
        "declared_paths": staged["declared_paths"],
    }
    drift = [key for key, value in expected.items() if baseline.get(key) != value]
    if drift:
        raise MaintError(
            "maintenance baseline no longer matches the staged delivery: "
            + ", ".join(drift)
            + "\nremove the stale ignored record and restart only after the tree is clean"
        )


def _landed_maint(repo: Path, iid: int) -> list[str]:
    proc = git(
        repo, "log", "--format=%H %s", "--extended-regexp", f"--grep=^Refs #{iid}$",
    )
    if proc.returncode != 0:
        raise MaintError(proc.stderr.strip() or "cannot inspect maintenance history")
    return [line for line in proc.stdout.splitlines() if " maint(" in f" {line}"]


def cmd_start(args: argparse.Namespace) -> int:
    repo = repo_root()
    owner = project_root(repo, args.project)
    staged = validate_staged(repo, owner, args.wi)
    paths = dirty_paths(repo)
    if paths:
        raise MaintError(
            "start needs a clean working tree; existing changes cannot be attributed:\n"
            + "\n".join(f"  {path}" for path in paths[:20])
        )
    landed = _landed_maint(repo, args.wi)
    if landed:
        raise MaintError(
            f"the MAINT phase for #{args.wi} already landed:\n"
            + "\n".join(f"  {line}" for line in landed)
        )
    value: dict[str, Any] = {
        "schema": SCHEMA_VERSION,
        "iid": args.wi,
        "project": args.project,
        "owner": staged["owner"],
        "type": staged["type"],
        "flow": staged["flow"],
        "base": head_sha(repo),
        "body_sha256": staged["body_sha256"],
        "receipt_sha256": staged["receipt_sha256"],
        "gates": staged["gates"],
        "declared_paths": staged["declared_paths"],
        "records": {"before": {}, "after": {}},
    }
    path = record_path(repo, args.wi)
    if path.exists():
        existing = load_baseline(repo, args.wi)
        comparable = dict(existing)
        comparable["records"] = {"before": {}, "after": {}}
        if existing.get("records") != {"before": {}, "after": {}} or comparable != value:
            raise MaintError(
                f"a maintenance record already exists at {path}; it was not overwritten"
            )
        print(f"maintenance baseline already open: {path}")
    else:
        _save_record(path, value)
        print(f"opened MAINT for #{args.wi}: type:{staged['type']}")
        print(f"baseline: {value['base']}")
        print(f"record: {path}")
    if staged["type"] == "refactor":
        print(
            "next.command: review and run each declared behavior gate outside "
            "maint.py, then record its exact command, exit code, and output file"
        )
    else:
        print(
            "next.command: make only the GHAN-declared maintenance change, then "
            "review and run each declared gate outside maint.py and record its "
            "exact command, exit code, and output file"
        )
    return 0


def cmd_record(args: argparse.Namespace) -> int:
    repo = repo_root()
    owner = project_root(repo, args.project)
    staged = validate_staged(repo, owner, args.wi)
    baseline = load_baseline(repo, args.wi)
    _record_identity_matches(baseline, staged, args.project)
    if head_sha(repo) != baseline.get("base"):
        raise MaintError(
            f"HEAD moved after start: baseline {baseline.get('base')}, now {head_sha(repo)}"
        )
    if _landed_maint(repo, args.wi):
        raise MaintError(f"MAINT for #{args.wi} is already committed")
    if args.command not in baseline["gates"]:
        raise MaintError(
            "record command is not an exact Acceptance command:\n"
            f"  {args.command}\n"
            "declared:\n" + "\n".join(f"  {command}" for command in baseline["gates"])
        )
    paths = dirty_paths(repo)
    if args.when == "before":
        if baseline["type"] != "refactor":
            raise MaintError("only type:refactor records behavior gates before the change")
        if paths:
            raise MaintError(
                "a before record needs the clean baseline tree; changed paths:\n"
                + "\n".join(f"  {path}" for path in paths)
            )
        tree = baseline["base"]
    else:
        if not paths:
            raise MaintError("an after record needs a maintenance diff")
        if baseline["type"] == "refactor" and args.command not in baseline["records"]["before"]:
            raise MaintError(
                f"record the before result for `{args.command}` before its after result"
            )
        tree = change_digest(repo, baseline, paths)
    output = Path(args.output_file)
    if not output.is_file():
        raise MaintError(f"--output-file is not a file: {output}")
    output_bytes = output.read_bytes()
    evidence = {
        "command": args.command,
        "exit": args.exit,
        "output_sha256": sha256_bytes(output_bytes),
        "output_bytes": len(output_bytes),
        "head": baseline["base"],
        "tree": tree,
    }
    records = baseline.setdefault("records", {}).setdefault(args.when, {})
    existing = records.get(args.command)
    if existing is not None and existing != evidence:
        raise MaintError(
            f"{args.when} evidence for `{args.command}` already exists and differs; "
            "it was not overwritten"
        )
    records[args.command] = evidence
    _save_record(record_path(repo, args.wi), baseline)
    print(
        f"recorded {args.when}: exit={args.exit} "
        f"output_sha256={evidence['output_sha256']} command={args.command}"
    )
    print("the output bytes were read for their digest and were not retained")
    print(
        f"next.command: {AW_CLI} maint --project {args.project} verify {args.wi}"
    )
    return 0


def gate_errors(baseline: dict[str, Any], current_digest: str) -> list[str]:
    kind = baseline["type"]
    gates = list(baseline["gates"])
    records = baseline.get("records") or {}
    required = ("before", "after") if kind == "refactor" else ("after",)
    errors: list[str] = []
    for when in required:
        stage = records.get(when) or {}
        missing = [command for command in gates if command not in stage]
        extra = [command for command in stage if command not in gates]
        if missing:
            errors.append(
                f"missing {when} record(s): " + ", ".join(f"`{c}`" for c in missing)
            )
        if extra:
            errors.append(
                f"undeclared {when} record(s): " + ", ".join(f"`{c}`" for c in extra)
            )
        for command in gates:
            evidence = stage.get(command)
            if not isinstance(evidence, dict):
                continue
            if evidence.get("command") != command:
                errors.append(f"{when} record does not preserve exact command `{command}`")
            if evidence.get("exit") != 0:
                errors.append(
                    f"{when} `{command}` exited {evidence.get('exit')}, not 0"
                )
            output_sha = str(evidence.get("output_sha256") or "")
            if not re.fullmatch(r"[0-9a-f]{64}", output_sha):
                errors.append(f"{when} `{command}` has no valid output sha256")
            expected_tree = baseline["base"] if when == "before" else current_digest
            if evidence.get("tree") != expected_tree:
                errors.append(
                    f"{when} `{command}` describes a different tree; record it again"
                )
            if evidence.get("head") != baseline["base"]:
                errors.append(f"{when} `{command}` describes a different baseline")
    # A non-refactor type has no legitimate before population.  Refuse one
    # rather than silently omitting it from the accepted evidence digest.
    if kind != "refactor" and (records.get("before") or {}):
        errors.append(f"type:{kind} must not carry manufactured before evidence")
    return errors


def verify_state(args: argparse.Namespace) -> tuple[Check, Path, dict[str, Any]]:
    chk = Check()
    repo = repo_root()
    owner = project_root(repo, args.project)
    try:
        staged = validate_staged(repo, owner, args.wi)
        chk.add("PASS", "P1 staged delivery", f"valid type:{staged['type']} body and receipt")
    except MaintError as exc:
        chk.add("FAIL", "P1 staged delivery", str(exc))
        return chk, repo, {}
    try:
        baseline = load_baseline(repo, args.wi)
        _record_identity_matches(baseline, staged, args.project)
        now = head_sha(repo)
        if now != baseline.get("base"):
            raise MaintError(f"baseline is {baseline.get('base')}; HEAD is now {now}")
        if _landed_maint(repo, args.wi):
            raise MaintError(f"MAINT for #{args.wi} is already committed")
        chk.add("PASS", "P2 baseline", f"HEAD remains {now}")
    except MaintError as exc:
        chk.add("FAIL", "P2 baseline", str(exc))
        return chk, repo, {}

    paths = dirty_paths(repo)
    if not paths:
        chk.add("FAIL", "C0 maintenance diff", "nothing differs from HEAD")
        return chk, repo, {"baseline": baseline, "paths": []}
    scope = type_scope_errors(repo, baseline, paths)
    if scope:
        chk.add("FAIL", f"C0 type:{baseline['type']} scope", "\n".join(scope))
    else:
        chk.add(
            "PASS", f"C0 type:{baseline['type']} scope",
            f"all {len(paths)} changed path(s) satisfy the declared type boundary",
        )

    digest = change_digest(repo, baseline, paths)
    evidence_errors = gate_errors(baseline, digest)
    if evidence_errors:
        chk.add("FAIL", "C1 controller-run gates", "\n".join(evidence_errors))
    else:
        stages = 2 if baseline["type"] == "refactor" else 1
        chk.add(
            "PASS", "C1 controller-run gates",
            f"{len(baseline['gates'])} exact command(s) have {stages} required record stage(s)",
        )

    records = baseline.get("records") or {}
    gate_digest = canonical_digest({
        "gates": baseline["gates"],
        "before": records.get("before") or {},
        "after": records.get("after") or {},
    })
    contract_digest = canonical_digest({
        "iid": baseline["iid"],
        "type": baseline["type"],
        "base": baseline["base"],
        "body_sha256": baseline["body_sha256"],
        "declared_paths": baseline["declared_paths"],
        "gates": baseline["gates"],
        "gate_digest": gate_digest,
    })
    return chk, repo, {
        "baseline": baseline,
        "paths": paths,
        "change_digest": digest,
        "gate_digest": gate_digest,
        "contract_digest": contract_digest,
    }


def cmd_verify(args: argparse.Namespace) -> int:
    chk, _repo, state = verify_state(args)
    print(f"maintenance admissibility: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnext.command: fix the FAIL rows, then re-run verify")
        return 1
    baseline = state["baseline"]
    print()
    print(
        f"type:{baseline['type']} is admissible without manufactured red evidence."
    )
    print(f"Maint-Contract: {state['contract_digest']}")
    print(f"Maint-Change-Digest: {state['change_digest']}")
    print(f"next.command: {AW_CLI} maint --project {args.project} commit {args.wi}")
    return 0


def _message(args: argparse.Namespace, state: dict[str, Any]) -> str:
    baseline = state["baseline"]
    records = baseline.get("records") or {}
    evidence_lines: list[str] = []
    for when in ("before", "after"):
        for command in baseline["gates"]:
            evidence = (records.get(when) or {}).get(command)
            if not evidence:
                continue
            evidence_lines.append(
                f"  {when} exit={evidence['exit']} output={evidence['output_sha256']} "
                f"command={json.dumps(command)}"
            )
    body = "\n".join(evidence_lines)
    trailers = [
        f"Maint-Type: {baseline['type']}",
        f"Maint-Base: {baseline['base']}",
        f"Maint-Gates: {state['gate_digest']}",
        f"Maint-Contract: {state['contract_digest']}",
        f"Maint-Change-Digest: {state['change_digest']}",
    ]
    return (
        f"maint(wi-{args.wi}): land type:{baseline['type']} maintenance\n\n"
        f"Controller-run gate evidence:\n{body}\n\n"
        f"Refs #{args.wi}\n\n"
        + "\n".join(trailers)
        + "\n"
    )


def _print_followups(args: argparse.Namespace, sha: str, digest: str) -> None:
    print(
        f"next.command: {AW_CLI} change lifecycle {args.wi} --leg {PHASE} "
        f"--commit {sha} --digest {digest}"
    )
    print(f"after.lifecycle.command: {AW_CLI} change close {args.wi}")


def cmd_commit(args: argparse.Namespace) -> int:
    chk, repo, state = verify_state(args)
    print(f"maintenance commit gate: #{args.wi}")
    chk.report()
    if chk.failed:
        print("\nnothing was committed; fix the FAIL rows, then re-run commit")
        return 1
    message = _message(args, state)
    paths = state["paths"]
    if args.dry_run:
        print("\n-- would commit, exactly these paths ------------------------")
        for path in paths:
            print(f"  {path}")
        print("-- message -------------------------------------------------")
        print(message)
        print(
            "next.command: run this commit verb without --dry-run; only a "
            "successful commit can supply the lifecycle command"
        )
        return 0

    add = git(repo, "add", "--", *paths)
    if add.returncode != 0:
        print(add.stderr or add.stdout)
        return add.returncode
    committed = git(repo, "commit", "-m", message, "--", *paths)
    print(committed.stdout or committed.stderr)
    if committed.returncode != 0:
        return committed.returncode
    sha = head_sha(repo)
    print(f"Maint-Commit: {sha}")
    record_path(repo, args.wi).unlink(missing_ok=True)
    _print_followups(args, sha, state["change_digest"])
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="maint.py", description=__doc__.splitlines()[0],
    )
    parser.add_argument(
        "--project", required=True,
        help="one project name under apps/ or libs/; must precede the verb",
    )
    sub = parser.add_subparsers(dest="verb", required=True)
    wi = argparse.ArgumentParser(add_help=False)
    wi.add_argument("wi", type=int, help="delivery issue iid")

    command = sub.add_parser(
        "start", parents=[wi], help="pin a clean baseline for one maintenance delivery",
    )
    command.set_defaults(func=cmd_start)

    command = sub.add_parser(
        "record", parents=[wi],
        help="record one exact controller-run command without executing or retaining it",
    )
    command.add_argument("--when", required=True, choices=("before", "after"))
    command.add_argument("--command", required=True, help="exact backticked Acceptance command")
    command.add_argument("--exit", required=True, type=int, dest="exit")
    command.add_argument("--output-file", required=True)
    command.set_defaults(func=cmd_record)

    command = sub.add_parser(
        "verify", parents=[wi], help="validate type scope and recorded gate evidence",
    )
    command.set_defaults(func=cmd_verify)

    command = sub.add_parser(
        "commit", parents=[wi], help="verify and commit exactly the measured paths",
    )
    command.add_argument("--dry-run", action="store_true")
    command.set_defaults(func=cmd_commit)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return args.func(args)
    except MaintError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
