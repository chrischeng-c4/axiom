#!/usr/bin/env python3
"""plane_b_lock.py — Plane-B Lock Tooling (Generator + Drift Linter + Backfill)

Manages provenance, lock verification, generator emission, and backfilling for
Mamba's cpython_ported integration test suite under `tests/cpython_ported/`
(see `tests/cpython_ported/harness.rs` for `jit_capture`/`assert_output`).

Usage:
    python3 tools/plane_b_lock.py lint [--verbose]
    python3 tools/plane_b_lock.py generate <fixture_path>... [--output <file.rs>]
    python3 tools/plane_b_lock.py backfill [--dry-run]
"""

from __future__ import annotations

import argparse
import bisect
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# Repo Geometry
TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parent
CPYTHON_DIR = MAMBA_DIR / "tests" / "cpython"
LAST_GATE_PATH = CPYTHON_DIR / ".cache" / "conformance" / "last_gate.json"
PORTED_DIR = MAMBA_DIR / "tests" / "cpython_ported"

def safe_rel_path(path: Path, base: Path = MAMBA_DIR) -> str:
    try:
        return path.resolve().relative_to(base.resolve()).as_posix()
    except Exception:
        try:
            return path.relative_to(base).as_posix()
        except Exception:
            p_str = path.resolve().as_posix()
            b_str = base.resolve().as_posix()
            if p_str.startswith(b_str):
                return p_str[len(b_str):].lstrip("/")
            return path.as_posix()


PROV_RE = re.compile(r"///\s*Ported from\s+(.+)$")
MOD_PROV_RE = re.compile(r"//!\s*Ported from\s+(.+)$")
FN_TEST_RE = re.compile(r"^(?:pub\s+)?fn\s+(test_\w+)\s*\(", re.MULTILINE)
MAMBA_BLOCK_RE = re.compile(r"# /// script\n(.*?)\n# ///", re.DOTALL)
NL_RE = re.compile(r"\n")

FIXTURE_CONTENT_CACHE: dict[str, str] = {}


def normalize_code(code: str) -> str:
    """Normalize python source code for exact semantic content comparison."""
    return code.replace("\r\n", "\n").strip()


def extract_jit_capture_code(fn_text: str) -> str:
    """Extract embedded Python code inside `jit_capture(...)` or `run_type_wall_fixture(...)`."""
    m = re.search(r"(?:jit_capture|run_type_wall_fixture)\s*\(\s*r(#*)\"", fn_text)
    if not m:
        m2 = re.search(r"(?:jit_capture|run_type_wall_fixture)\s*\(\s*\"(.*?)\"\s*\)", fn_text, re.DOTALL)
        return m2.group(1) if m2 else ""
    hashes = m.group(1)
    closing_delim = '"' + hashes
    start_idx = m.end()
    end_idx = fn_text.find(closing_delim, start_idx)
    if end_idx != -1:
        return fn_text[start_idx:end_idx]
    return ""


def load_cpython_fixtures(load_contents: bool = False) -> tuple[set[str], dict[str, list[str]], dict[str, str]]:
    """Index python fixtures under `tests/cpython/`.

    Returns:
        - fixtures_set: set of relative paths
        - filename_to_paths: filename -> list of rel_paths
        - content_to_path: normalized content -> rel_path (only populated if load_contents=True)
    """
    fixtures_set: set[str] = set()
    filename_to_paths: dict[str, list[str]] = {}
    content_to_path: dict[str, str] = {}

    if CPYTHON_DIR.exists():
        for root, dirs, files in os.walk(CPYTHON_DIR):
            if ".cache" in root or "__pycache__" in root:
                continue
            for f in files:
                if f.endswith(".py"):
                    full_p = Path(root) / f
                    rel_p = safe_rel_path(full_p)
                    fixtures_set.add(rel_p)
                    filename_to_paths.setdefault(f, []).append(rel_p)

                    if load_contents:
                        try:
                            content = full_p.read_text(encoding="utf-8", errors="ignore")
                        except OSError:
                            continue
                        FIXTURE_CONTENT_CACHE[rel_p] = content
                        norm = normalize_code(content)
                        if norm not in content_to_path:
                            content_to_path[norm] = rel_p

    return fixtures_set, filename_to_paths, content_to_path


def read_fixture_content(rel_p: str) -> str | None:
    """Read fixture content on demand with caching."""
    if rel_p in FIXTURE_CONTENT_CACHE:
        return FIXTURE_CONTENT_CACHE[rel_p]
    full_p = MAMBA_DIR / rel_p
    if not full_p.exists():
        return None
    try:
        content = full_p.read_text(encoding="utf-8", errors="ignore")
        FIXTURE_CONTENT_CACHE[rel_p] = content
        return content
    except OSError:
        return None


def load_last_gate(fixtures_set: set[str]) -> set[str]:
    """Load passing fixture paths from `last_gate.json`.

    Returns set of relative paths that have verdict PASS.
    """
    if not LAST_GATE_PATH.exists():
        return set()

    try:
        data = json.loads(LAST_GATE_PATH.read_text(encoding="utf-8"))
    except Exception:
        return set()

    non_pass_paths = {item["path"] for item in data.get("non_pass", [])}
    return fixtures_set - non_pass_paths


def resolve_oracle_python() -> str:
    """Prefer the pinned CPython 3.12 oracle env the sweep harness uses
    (`tests/cpython/.cache/oracle-env`); fall back to plain `python3`."""
    override = os.environ.get("MAMBA_ORACLE_PYTHON", "").strip()
    if override:
        return override
    oracle_env = CPYTHON_DIR / ".cache" / "oracle-env" / "bin" / "python3"
    if oracle_env.is_file():
        return str(oracle_env)
    return "python3"


def run_python_oracle(fixture_path: Path) -> tuple[str | None, str]:
    """Actually execute `fixture_path` with the CPython oracle and return its
    stdout as `(stdout, "")`. On failure (non-zero exit, timeout, launch
    error) returns `(None, error_message)` — the caller must refuse to
    generate a test for that fixture rather than emit a blind assertion-free
    one (CONTRIBUTING.md four-plane test doctrine)."""
    python = resolve_oracle_python()
    try:
        proc = subprocess.run(
            [python, str(fixture_path)],
            capture_output=True, text=True, timeout=30,
        )
    except subprocess.TimeoutExpired:
        return None, f"{python} {fixture_path} timed out after 30s"
    except OSError as e:
        return None, f"failed to launch {python}: {e}"

    if proc.returncode != 0:
        return None, (
            f"{python} {fixture_path} exited {proc.returncode}\n"
            f"--- stderr ---\n{proc.stderr}"
        )
    return proc.stdout, ""


def resolve_provenance_path(raw: str, fixtures_set: set[str], filename_to_paths: dict[str, list[str]]) -> str | None:
    """Clean up and resolve a provenance path string from a doc comment."""
    cleaned = raw.strip()
    if cleaned.startswith("`") and cleaned.endswith("`"):
        cleaned = cleaned[1:-1]
    if cleaned.startswith('"') and cleaned.endswith('"'):
        cleaned = cleaned[1:-1]
    if cleaned.startswith("'") and cleaned.endswith("'"):
        cleaned = cleaned[1:-1]
    if cleaned.endswith(".") and not cleaned.endswith("..") and not cleaned.endswith(".py"):
        cleaned = cleaned[:-1].rstrip()
    if cleaned.startswith("`") and cleaned.endswith("`"):
        cleaned = cleaned[1:-1]

    cleaned = cleaned.strip()
    if not cleaned:
        return None

    if cleaned in fixtures_set:
        return cleaned

    if cleaned.startswith("tests/cpython/"):
        return cleaned

    if cleaned.startswith("cpython/"):
        cand = "tests/" + cleaned
        if cand in fixtures_set:
            return cand
        return cand

    if cleaned.startswith("Lib/test/") or cleaned.startswith("Lib/"):
        fname = os.path.basename(cleaned)
        if fname in filename_to_paths:
            paths = filename_to_paths[fname]
            if len(paths) == 1:
                return paths[0]
            for p in paths:
                if p.endswith(fname):
                    return p
        return cleaned

    if cleaned.startswith("commit") or "commit " in cleaned or cleaned.startswith("CPython"):
        return cleaned

    cand = "tests/cpython/" + cleaned
    if cand in fixtures_set:
        return cand

    fname = os.path.basename(cleaned)
    if fname in filename_to_paths:
        paths = filename_to_paths[fname]
        if len(paths) == 1:
            return paths[0]
        for p in paths:
            if p.endswith(cleaned):
                return p

    return cand


def extract_metadata_block_path(py_code: str, fixtures_set: set[str]) -> str | None:
    """Attempt to construct fixture path from embedded [tool.mamba] block metadata."""
    if not py_code:
        return None
    block_m = MAMBA_BLOCK_RE.search(py_code)
    if not block_m:
        return None
    block_txt = block_m.group(1)

    dim = re.search(r'dimension\s*=\s*"([^"]+)"', block_txt)
    bkt = re.search(r'bucket\s*=\s*"([^"]+)"', block_txt)
    lib = re.search(r'lib\s*=\s*"([^"]+)"', block_txt)
    case = re.search(r'case\s*=\s*"([^"]+)"', block_txt)

    if dim and bkt and case:
        dim_v = dim.group(1)
        bkt_v = bkt.group(1)
        case_v = case.group(1)
        lib_v = lib.group(1) if lib else ""

        cands = []
        if lib_v:
            cands.append(f"tests/cpython/{dim_v}/{bkt_v}/{lib_v}/{case_v}.py")
        cands.append(f"tests/cpython/{dim_v}/{bkt_v}/{case_v}.py")

        for cand in cands:
            if cand in fixtures_set:
                return cand

    return None


def extract_module_header_path(content: str) -> str | None:
    """Extract provenance path from file-level module doc comment (//! Ported from ...)."""
    lines = content.splitlines()[:20]
    for line in lines:
        if "//!" in line and ("Ported from" in line or "ported from" in line):
            m = MOD_PROV_RE.search(line)
            raw = m.group(1).strip() if m else line.replace("//!", "").strip()

            m_paren = re.search(r"\(([^)]+)\)", raw)
            if m_paren:
                inner = m_paren.group(1).strip()
                inner = inner.split("—")[0].split(":")[0].strip()
                if inner:
                    return inner

            cleaned = raw.replace("CPython 3.12.0 tag", "").replace("CPython", "").strip()
            cleaned = cleaned.lstrip("(").rstrip(")").split("—")[0].split(":")[0].strip()
            if cleaned:
                return cleaned
    return None


def parse_rs_file_functions(file_path: Path) -> list[dict]:
    """Parse a single Rust test file for test functions, doc comments, and jit_capture python code."""
    content = file_path.read_text(encoding="utf-8", errors="ignore")
    lines = content.splitlines()

    line_starts = [0] + [m.start() + 1 for m in NL_RE.finditer(content)]

    fn_matches = list(FN_TEST_RE.finditer(content))
    test_funcs = []

    for idx, m in enumerate(fn_matches):
        fn_name = m.group(1)
        fn_start = m.start()

        line_no = bisect.bisect_right(line_starts, fn_start) - 1

        prov_raw = None
        back_start = max(0, line_no - 15)
        for lno in range(line_no - 1, back_start - 1, -1):
            prev_line = lines[lno].strip()
            if prev_line.startswith("///"):
                if "Ported from" in prev_line:
                    pm = PROV_RE.search(prev_line)
                    if pm:
                        prov_raw = pm.group(1).strip()
                        break
            elif prev_line.startswith("#[test]") or prev_line.startswith("#[") or prev_line == "":
                continue
            else:
                break

        next_start = fn_matches[idx + 1].start() if idx + 1 < len(fn_matches) else len(content)
        fn_text = content[fn_start:next_start]
        py_code = extract_jit_capture_code(fn_text)

        test_funcs.append({
            "fn_name": fn_name,
            "line_no": line_no,
            "prov_raw": prov_raw,
            "py_code": py_code,
            "fn_text": fn_text,
            "file_path": file_path,
        })

    return test_funcs


def run_linter(verbose: bool = False) -> int:
    """Run provenance drift linter over all plane-B test files."""
    fixtures_set, filename_to_paths, _ = load_cpython_fixtures(load_contents=False)
    pass_set = load_last_gate(fixtures_set)

    counts = {
        "in-sync": 0,
        "stale-lock": 0,
        "copy-corrupt": 0,
        "missing-provenance": 0,
        "external-provenance": 0,
    }

    details = {k: [] for k in counts}

    for root, dirs, files in os.walk(PORTED_DIR):
        for f in sorted(files):
            if f.endswith(".rs"):
                fpath = Path(root) / f
                funcs = parse_rs_file_functions(fpath)
                rel_fpath = safe_rel_path(fpath)

                for fn in funcs:
                    prov_raw = fn["prov_raw"]
                    py_code = fn["py_code"]
                    fn_name = fn["fn_name"]

                    if not prov_raw:
                        counts["missing-provenance"] += 1
                        details["missing-provenance"].append((rel_fpath, fn_name))
                        continue

                    resolved = resolve_provenance_path(prov_raw, fixtures_set, filename_to_paths)

                    if not resolved:
                        counts["stale-lock"] += 1
                        details["stale-lock"].append((rel_fpath, fn_name, f"missing-fixture: {prov_raw}"))
                        continue

                    if resolved not in fixtures_set:
                        if resolved.startswith("tests/cpython/"):
                            # Looks like a local fixture path but the file is
                            # actually gone — a genuinely broken reference.
                            counts["stale-lock"] += 1
                            details["stale-lock"].append((rel_fpath, fn_name, f"missing-fixture: {prov_raw}"))
                        else:
                            # Doesn't map into the local `tests/cpython/`
                            # corpus at all (raw `Lib/test/...` upstream
                            # paths, commit references, free text, etc. —
                            # typically old baseline references to CPython
                            # upstream). Not a local-content drift signal:
                            # there's nothing in-repo to compare against.
                            counts["external-provenance"] += 1
                            details["external-provenance"].append(
                                (rel_fpath, fn_name, f"unresolved: {prov_raw} -> {resolved}")
                            )
                        continue

                    # `resolved` names a real fixture file under
                    # tests/cpython/ — this is the only case where an
                    # actual content comparison happens.
                    fixture_code = read_fixture_content(resolved)
                    if fixture_code is None:
                        counts["stale-lock"] += 1
                        details["stale-lock"].append((rel_fpath, fn_name, f"unreadable-fixture: {resolved}"))
                    elif normalize_code(py_code) != normalize_code(fixture_code):
                        counts["copy-corrupt"] += 1
                        details["copy-corrupt"].append((rel_fpath, fn_name, resolved))
                    elif resolved not in pass_set:
                        counts["stale-lock"] += 1
                        details["stale-lock"].append((rel_fpath, fn_name, f"non-pass: {resolved}"))
                    else:
                        counts["in-sync"] += 1

    print("Plane-B Provenance Drift Linter Results:")
    print(f"  in-sync:            {counts['in-sync']}")
    print(f"  stale-lock:         {counts['stale-lock']}")
    print(f"  copy-corrupt:       {counts['copy-corrupt']}")
    print(f"  missing-provenance: {counts['missing-provenance']}")
    print(f"  external-provenance:{counts['external-provenance']:>5}")

    if verbose or counts["missing-provenance"] > 0 or counts["copy-corrupt"] > 0:
        for cat in ["copy-corrupt", "missing-provenance", "stale-lock", "external-provenance"]:
            if details[cat]:
                print(f"\n--- {cat.upper()} ({len(details[cat])}) ---")
                for item in details[cat][:30]:
                    print(" ", item)

    # CI invokable: exit non-zero if copy-corrupt > 0 or missing-provenance > 0
    if counts["copy-corrupt"] > 0 or counts["missing-provenance"] > 0:
        return 1
    return 0


def generate_tests(fixture_paths: list[str], output_path: str | None = None) -> int:
    """Generate plane-B test functions — with a real `assert_output` against
    actual CPython oracle stdout — for given Python fixtures.

    Each fixture is executed with `python3 <fixture>` (preferring the pinned
    `tests/cpython/.cache/oracle-env` CPython 3.12) to capture its expected
    stdout. A fixture whose oracle run exits non-zero (or times out / fails
    to launch) is a hard error for that fixture: it is NOT generated — the
    four-plane test doctrine forbids emitting a blind, assertion-free test
    (see CONTRIBUTING.md "Test architecture (four planes)").
    """
    fixtures_set, _, _ = load_cpython_fixtures(load_contents=False)

    generated_blocks = []
    failed = []

    for fpath_raw in fixture_paths:
        p = Path(fpath_raw)
        rel_p = safe_rel_path(p) if p.is_absolute() else fpath_raw
        if not rel_p.startswith("tests/cpython/"):
            rel_p = "tests/cpython/" + rel_p.lstrip("/")

        full_p = MAMBA_DIR / rel_p
        code = read_fixture_content(rel_p)
        if code is None:
            if full_p.exists():
                code = full_p.read_text(encoding="utf-8")
            else:
                print(f"Error: Fixture {rel_p} not found", file=sys.stderr)
                failed.append(rel_p)
                continue

        expected_stdout, err = run_python_oracle(full_p)
        if expected_stdout is None:
            print(f"Error: oracle run failed for {rel_p}, refusing to generate:\n{err}", file=sys.stderr)
            failed.append(rel_p)
            continue

        parts = rel_p.replace("tests/cpython/", "").replace(".py", "").split("/")
        slug_parts = [re.sub(r"\W+", "_", part) for part in parts]
        fn_slug = "_".join(slug_parts)

        block = f'''/// Ported from `{rel_p}`.
#[test]
fn test_gen_{fn_slug}() {{
    let out = jit_capture(r###"{code}"###);
    assert_output(&out, r###"{expected_stdout}"###);
}}'''
        generated_blocks.append(block)

    if not generated_blocks:
        print("Error: no fixtures generated (all failed the oracle run)", file=sys.stderr)
        return 1

    output_text = "\n\n".join(generated_blocks) + "\n"

    if output_path:
        out_p = Path(output_path)
        out_p.parent.mkdir(parents=True, exist_ok=True)
        if out_p.exists():
            existing = out_p.read_text(encoding="utf-8")
            out_p.write_text(existing.rstrip() + "\n\n" + output_text, encoding="utf-8")
        else:
            # Matches the import style used throughout tests/cpython_ported/
            # (e.g. tests/cpython_ported/stdlib/json.rs) for a depth-2
            # submodule reaching the sibling `harness` module.
            header = "use super::super::harness::*;\n\n"
            out_p.write_text(header + output_text, encoding="utf-8")
        print(f"Generated {len(generated_blocks)} test fixtures into {output_path}")
    else:
        print(output_text)

    if failed:
        print(f"\n{len(failed)} fixture(s) skipped (oracle run failed, see errors above): {failed}", file=sys.stderr)

    return 0


def run_backfill(dry_run: bool = False) -> int:
    """Perform provenance backfill pass for missing-provenance test functions."""
    print("Indexing CPython fixtures for backfill pass...")
    fixtures_set, filename_to_paths, content_to_path = load_cpython_fixtures(load_contents=True)

    missing_count = 0
    filled_count = 0
    residue_list = []

    for root, dirs, files in os.walk(PORTED_DIR):
        for f in sorted(files):
            if not f.endswith(".rs"):
                continue
            fpath = Path(root) / f
            content = fpath.read_text(encoding="utf-8", errors="ignore")
            lines = content.splitlines()

            funcs = parse_rs_file_functions(fpath)
            if not funcs:
                continue

            modified = False
            lines_inserted = 0

            mod_header_path = extract_module_header_path(content)

            for fn in funcs:
                if fn["prov_raw"]:
                    continue

                missing_count += 1
                line_no = fn["line_no"] + lines_inserted
                py_code = fn["py_code"]
                fn_name = fn["fn_name"]

                norm_code = normalize_code(py_code)

                # Attempt 1: Content match
                target_path = content_to_path.get(norm_code)

                # Attempt 2: Metadata block match
                if not target_path:
                    target_path = extract_metadata_block_path(py_code, fixtures_set)

                # Attempt 3: Module header path
                if not target_path and mod_header_path:
                    target_path = mod_header_path

                if target_path:
                    insert_line = line_no
                    if line_no > 0 and lines[line_no - 1].strip().startswith("#[test]"):
                        insert_line = line_no - 1

                    indent = " " * (len(lines[insert_line]) - len(lines[insert_line].lstrip()))
                    comment_line = f"{indent}/// Ported from `{target_path}`."
                    lines.insert(insert_line, comment_line)
                    lines_inserted += 1
                    modified = True
                    filled_count += 1
                else:
                    rel_fpath = safe_rel_path(fpath)
                    residue_list.append((rel_fpath, fn_name))

            if modified and not dry_run:
                fpath.write_text("\n".join(lines) + "\n", encoding="utf-8")

    remaining = missing_count - filled_count
    print(f"Backfill Pass Summary:")
    print(f"  Missing provenance total: {missing_count}")
    print(f"  Successfully backfilled:  {filled_count}")
    print(f"  Remaining residue count:  {remaining}")

    if residue_list:
        print("\nResidue functions (missing provenance):")
        for item in residue_list[:50]:
            print(f"  {item[0]} :: {item[1]}")
        if len(residue_list) > 50:
            print(f"  ... and {len(residue_list) - 50} more")

    return 0


RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try"
}


def sanitize_ident(name: str) -> str:
    s = re.sub(r"\W+", "_", name).strip("_")
    if not s:
        s = "_root"
    if s[0].isdigit() or s in RUST_KEYWORDS:
        s = f"_{s}"
    return s


def make_raw_rs_string(text: str) -> str:
    max_h = 0
    for m in re.finditer(r'#*"#*', text):
        cnt = m.group(0).count('#')
        if cnt > max_h:
            max_h = cnt
    hashes = "#" * max(3, max_h + 1)
    return f'r{hashes}"{text}"{hashes}'


def update_mod_rs(mod_rs_path: Path, active_mods: list[str]) -> None:
    mod_rs_path.parent.mkdir(parents=True, exist_ok=True)
    existing_entries: dict[str, str | None] = {}
    non_mod_lines = []

    if mod_rs_path.exists():
        content = mod_rs_path.read_text(encoding="utf-8")
        for line in content.splitlines():
            line_s = line.strip()
            if line_s and not line_s.startswith("pub mod ") and not line_s.startswith("#[path = "):
                non_mod_lines.append(line)

    for item in active_mods:
        m_name = sanitize_ident(item)
        if m_name != item:
            path_val = f"{item}/mod.rs" if not item.endswith(".rs") else item
            existing_entries[m_name] = path_val
        else:
            existing_entries[m_name] = None

    lines = list(non_mod_lines)
    for m in sorted(existing_entries.keys()):
        p_val = existing_entries[m]
        if p_val:
            lines.append(f'#[path = "{p_val}"]')
        lines.append(f"pub mod {m};")

    mod_rs_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def ensure_pub_mod(mod_rs_path: Path, mod_name: str) -> None:
    mod_rs_path.parent.mkdir(parents=True, exist_ok=True)
    existing_mods = set()
    non_mod_lines = []
    if mod_rs_path.exists():
        content = mod_rs_path.read_text(encoding="utf-8")
        for line in content.splitlines():
            line_s = line.strip()
            if line_s.startswith("pub mod ") and line_s.endswith(";"):
                m_name = line_s[len("pub mod "):-1].strip()
                existing_mods.add(m_name)
            elif line_s:
                non_mod_lines.append(line)
    existing_mods.add(mod_name)
    lines = non_mod_lines + [f"pub mod {m};" for m in sorted(existing_mods)]
    mod_rs_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_batch(dimension: str, bucket: str, plane: str = "b", out_root_str: str = "tests/cpython_ported/gen") -> int:
    out_root_p = Path(out_root_str)
    out_root = out_root_p if out_root_p.is_absolute() else MAMBA_DIR / out_root_p

    target_dir = CPYTHON_DIR / dimension / bucket
    if not target_dir.exists():
        print(f"Error: Target directory {target_dir} does not exist", file=sys.stderr)
        return 1

    fixtures_set, _, _ = load_cpython_fixtures(load_contents=False)
    pass_set = load_last_gate(fixtures_set)

    pass_fixtures = []
    for root, dirs, files in os.walk(target_dir):
        if ".cache" in root or "__pycache__" in root:
            continue
        for f in sorted(files):
            if f.endswith(".py"):
                full_p = Path(root) / f
                rel_p = safe_rel_path(full_p)
                if rel_p in pass_set:
                    pass_fixtures.append((rel_p, full_p))

    pass_fixtures.sort(key=lambda x: x[0])

    oracle_passed = []
    refused_dict: dict[str, str] = {}

    for rel_p, full_p in pass_fixtures:
        code = read_fixture_content(rel_p)
        if code is None:
            code = full_p.read_text(encoding="utf-8", errors="ignore")

        if plane == "c":
            has_strict_type = any(line.strip().startswith("# mamba-strict-type:") for line in code.splitlines())
            if not has_strict_type:
                refused_dict[rel_p] = "(no-directive)"
            else:
                oracle_passed.append((rel_p, full_p, code, None))
        else:
            expected_stdout, err = run_python_oracle(full_p)
            if expected_stdout is None:
                refused_dict[rel_p] = "(oracle-fail)"
            else:
                oracle_passed.append((rel_p, full_p, code, expected_stdout))

    fn_name_to_rel: dict[str, str] = {}
    group_blocks: dict[str, list[tuple[str, str]]] = {}

    for item in oracle_passed:
        if plane == "c":
            rel_p, full_p, code, _ = item
        else:
            rel_p, full_p, code, expected_stdout = item

        rel_to_bucket = full_p.relative_to(target_dir)
        parts = rel_to_bucket.parts
        group_raw = parts[0] if len(parts) > 1 else "_root"
        group = sanitize_ident(group_raw)

        parts = rel_p.replace("tests/cpython/", "").replace(".py", "").split("/")
        slug_parts = [re.sub(r"\W+", "_", part) for part in parts]
        fn_slug = "_".join(slug_parts)
        fn_name = f"test_gen_{fn_slug}"

        if group not in group_blocks:
            group_blocks[group] = []

        existing_names = {item[0] for item in group_blocks[group]}
        if fn_name in existing_names:
            h = hashlib.md5(rel_p.encode("utf-8")).hexdigest()[:6]
            fn_name = f"{fn_name}_{h}"

        fn_name_to_rel[fn_name] = rel_p

        code_str = make_raw_rs_string(code)

        if plane == "c":
            block = f'''/// Ported from `{rel_p}`.
#[test]
fn {fn_name}() {{
    let out = run_type_wall_fixture({code_str});
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {{out}}");
}}'''
        else:
            stdout_str = make_raw_rs_string(expected_stdout)
            block = f'''/// Ported from `{rel_p}`.
#[test]
fn {fn_name}() {{
    let out = jit_capture({code_str});
    assert_output(&out, {stdout_str});
}}'''

        group_blocks[group].append((fn_name, block))

    bucket_dir = out_root / dimension / bucket
    bucket_dir.mkdir(parents=True, exist_ok=True)
    header = "use super::super::super::super::harness::*;\n\n"

    def write_all():
        empty_groups = [g for g, items in group_blocks.items() if not items]
        for g in empty_groups:
            del group_blocks[g]
            g_file = bucket_dir / f"{g}.rs"
            if g_file.exists():
                g_file.unlink()

        for group, items in sorted(group_blocks.items()):
            items.sort(key=lambda x: x[0])
            group_file = bucket_dir / f"{group}.rs"
            content = header + "\n\n".join(b for _, b in items) + "\n"
            group_file.write_text(content, encoding="utf-8")

        ensure_pub_mod(PORTED_DIR / "mod.rs", "gen")
        active_dims = sorted(p.name for p in out_root.iterdir() if p.is_dir())
        update_mod_rs(out_root / "mod.rs", active_dims)
        active_buckets = sorted(p.name for p in (out_root / dimension).iterdir() if p.is_dir())
        update_mod_rs(out_root / dimension / "mod.rs", active_buckets)
        update_mod_rs(bucket_dir / "mod.rs", list(group_blocks.keys()))

    write_all()

    dim_mod_name = sanitize_ident(dimension)
    bucket_mod_name = sanitize_ident(bucket)
    cmd = [
        "cargo", "test", "-p", "mamba", "--test", "cpython_ported_integration",
        "--", f"gen::{dim_mod_name}::{bucket_mod_name}"
    ]
    proc = subprocess.run(cmd, cwd=MAMBA_DIR, capture_output=True, text=True)
    if proc.returncode != 0:
        failing_fns = set()
        in_failures = False
        for line in (proc.stdout + "\n" + proc.stderr).splitlines():
            line_s = line.strip()
            if line_s == "failures:":
                in_failures = True
                continue
            if in_failures:
                if not line_s:
                    continue
                if line_s.startswith("test result:"):
                    in_failures = False
                    continue
                if "::test_gen_" in line_s:
                    fn = line_s.split("::")[-1]
                    failing_fns.add(fn)

        if not failing_fns:
            for group, items in list(group_blocks.items()):
                group_cmd = [
                    "cargo", "test", "-p", "mamba", "--test", "cpython_ported_integration",
                    "--", f"gen::{dimension}::{bucket_mod_name}::{group}"
                ]
                g_proc = subprocess.run(group_cmd, cwd=MAMBA_DIR, capture_output=True, text=True)
                if g_proc.returncode != 0:
                    for line in (g_proc.stdout + "\n" + g_proc.stderr).splitlines():
                        line_s = line.strip()
                        if "::test_gen_" in line_s and ("panicked" in line_s or "failures:" in line_s or line_s.startswith("cpython_ported::")):
                            parts = line_s.split("::")
                            for p in parts:
                                if p.startswith("test_gen_"):
                                    failing_fns.add(p.split()[0])

                    for fn_name, _ in items:
                        fn_cmd = [
                            "cargo", "test", "-p", "mamba", "--test", "cpython_ported_integration",
                            "--", f"gen::{dimension}::{bucket_mod_name}::{group}::{fn_name}"
                        ]
                        fn_proc = subprocess.run(fn_cmd, cwd=MAMBA_DIR, capture_output=True, text=True)
                        if fn_proc.returncode != 0:
                            failing_fns.add(fn_name)

        for fn in failing_fns:
            if fn in fn_name_to_rel:
                rel_p = fn_name_to_rel[fn]
                refused_dict[rel_p] = "(postgen-fail)"
                for group in group_blocks:
                    group_blocks[group] = [item for item in group_blocks[group] if item[0] != fn]

        write_all()

    generated = sum(len(items) for items in group_blocks.values())
    refused = len(refused_dict)

    refused_list_formatted = [f"{rel_p} {cause}" for rel_p, cause in sorted(refused_dict.items())]
    print(f"generated={generated} refused={refused}")
    if refused_list_formatted:
        print(f"refused_list={refused_list_formatted}")

    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description="Plane-B/C lock tooling (Generator + Drift Linter + Backfill)")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Lint
    lint_parser = subparsers.add_parser("lint", help="Run provenance drift linter")
    lint_parser.add_argument("-v", "--verbose", action="store_true", help="Print details")

    # Generate
    gen_parser = subparsers.add_parser("generate", help="Generate plane-B test files from CPython fixtures")
    gen_parser.add_argument("fixtures", nargs="+", help="Python fixture file paths")
    gen_parser.add_argument("-o", "--output", help="Output Rust file path")

    # Backfill
    backfill_parser = subparsers.add_parser("backfill", help="Backfill missing provenance comments")
    backfill_parser.add_argument("--dry-run", action="store_true", help="Do not write changes")

    # Batch
    batch_parser = subparsers.add_parser("batch", help="Batch generate tests for a dimension/bucket")
    batch_parser.add_argument("--plane", choices=["b", "c"], default="b", help="Emission plane mode (b or c)")
    batch_parser.add_argument("--dimension", required=True, help="Dimension under tests/cpython")
    batch_parser.add_argument("--bucket", required=True, help="Bucket under dimension")
    batch_parser.add_argument("--out-root", default="tests/cpython_ported/gen", help="Output root directory")

    args = parser.parse_args()

    if args.command == "lint":
        sys.exit(run_linter(verbose=args.verbose))
    elif args.command == "generate":
        sys.exit(generate_tests(args.fixtures, output_path=args.output))
    elif args.command == "backfill":
        sys.exit(run_backfill(dry_run=args.dry_run))
    elif args.command == "batch":
        sys.exit(run_batch(dimension=args.dimension, bucket=args.bucket, plane=args.plane, out_root_str=args.out_root))


if __name__ == "__main__":
    main()

