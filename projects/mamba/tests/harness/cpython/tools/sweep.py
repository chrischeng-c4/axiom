#!/usr/bin/env python3.12
"""Incremental conformance sweep — re-run a SUBSET of fixtures in seconds.

The cargo conformance target re-runs all ~46k fixtures (~3 min wall) on every
invocation, which is the wrong instrument for a fix iteration loop: while
repairing a cluster you only need to know (1) do the fixtures I am fixing pass
now, and (2) did I regress anything that was passing. This tool answers both
in seconds by re-running an explicit subset with the SAME verdict semantics as
runner.rs:

  - `# mamba-xfail:` fixtures        -> XFAIL (skipped, like the runner)
  - `# RUN:` pipeline fixtures       -> SKIP  (owned by pipeline.rs)
  - bench/ fixtures                  -> SKIP  (owned by perf-pin tests)
  - /type/ dimension or `# mamba-strict-type:` -> STRICT_TYPE rules
    (compile-time type error or runtime `typeerror:` marker = PASS)
  - everything else: CPython oracle stdout must equal mamba stdout, with the
    oracle served from the SHARED D5.3 content-addressed cache
    (target/cpython-oracle-cache, same hash scheme as runner.rs) so a warm
    sweep costs only the mamba spawns (~47ms each, fully parallel).

This differs from gate_triage.py, which classifies the whole tree for triage
ranking but does not honor xfail/type-dimension semantics; sweep.py mirrors
the cargo gate verdict-for-verdict so its PASS/FAIL is the gate's PASS/FAIL.

Typical round flow:

  # once per round: refresh the stored failure set from a full sweep (~2 min)
  python3.12 tests/harness/cpython/tools/sweep.py --all --store

  # inner loop (seconds): re-check the cluster you are fixing
  python3.12 tests/harness/cpython/tools/sweep.py --failures --filter socket

  # regression canary (seconds): random sample of currently-passing fixtures
  python3.12 tests/harness/cpython/tools/sweep.py --sample 1500

  # explicit paths / lists also work
  python3.12 tests/harness/cpython/tools/sweep.py behavior/std-libs/socket
  python3.12 tests/harness/cpython/tools/sweep.py --list /tmp/cluster.txt

State: tests/cpython/.cache/sweep/failures.txt (one path per line, relative to
tests/cpython). `--store` merges this run into it: stored' = (stored - covered)
+ failed-in-this-run, so partial runs never resurrect or drop uncovered
entries. Limits parity note: runner.rs also caps child memory (RLIMIT_AS
1 GiB); macOS sh ulimit cannot express that, so only the CPU cap is applied
here — a runaway-allocation fixture can diverge between the two gates.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import random
import shlex
import shutil
import subprocess
import sys
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import harness_lib  # noqa: E402

TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]                       # projects/mamba
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"
STATE_DIR = FIXTURES_DIR / ".cache" / "sweep"
STATE_FILE = STATE_DIR / "failures.txt"
ORACLE_CACHE_DIR = (
    Path(os.environ["CARGO_TARGET_DIR"])
    if os.environ.get("CARGO_TARGET_DIR")
    else MAMBA_DIR.parents[1] / "target"
) / "cpython-oracle-cache"

DEFAULT_TIMEOUT = 30  # runner.rs DEFAULT_TIMEOUT_SECS

PASS, FAIL, DIVERGE, XFAIL, SKIP, INVALID = (
    "PASS", "FAIL", "DIVERGE", "XFAIL", "SKIP", "INVALID",
)
BAD = (FAIL, DIVERGE, INVALID)


def mamba_bin() -> str:
    if os.environ.get("MAMBA_BIN"):
        return os.environ["MAMBA_BIN"]
    candidate = MAMBA_DIR.parents[1] / "target" / "debug" / "mamba"
    if candidate.is_file():
        return str(candidate)
    return shutil.which("mamba") or "mamba"


def python3_bin() -> str:
    """Mirror harness_common.rs python3_bin() preference order."""
    override = os.environ.get("MAMBA_ORACLE_PYTHON", "").strip()
    if override:
        return override
    oracle_env = FIXTURES_DIR / ".cache" / "oracle-env" / "bin" / "python3"
    if oracle_env.is_file():
        return str(oracle_env)
    try:
        out = subprocess.run(
            ["python3", "-c", "import sys; print(sys.executable)"],
            capture_output=True, text=True, timeout=10, cwd="/tmp",
        )
        resolved = out.stdout.strip()
        if out.returncode == 0 and resolved:
            return resolved
    except Exception:  # noqa: BLE001
        pass
    return "python3"


def python_version_salt(python: str) -> str:
    try:
        out = subprocess.run([python, "-V"], capture_output=True, text=True, timeout=10)
        return (out.stdout.strip() or out.stderr.strip())
    except Exception:  # noqa: BLE001
        return ""


def oracle_cache_path(salt: str, src: str) -> Path:
    h = hashlib.sha256()
    h.update(salt.encode())
    h.update(b"\0v1\0")
    h.update(src.encode())
    hexd = h.hexdigest()
    return ORACLE_CACHE_DIR / hexd[:2] / hexd


def oracle_cache_put(cache_file: Path, stdout: bytes) -> None:
    cache_file.parent.mkdir(parents=True, exist_ok=True)
    tmp = cache_file.with_suffix(f".tmp{os.getpid()}")
    tmp.write_bytes(stdout)
    tmp.rename(cache_file)


def parse_directives(src: str) -> tuple[str | None, bool]:
    xfail, strict_type = None, False
    for line in src.splitlines():
        t = line.strip()
        if t.startswith("# mamba-xfail:"):
            xfail = t[len("# mamba-xfail:"):].strip()
        elif t.startswith("# mamba-strict-type:"):
            strict_type = True
    return xfail, strict_type


def run_mamba(bin_path: str, fixture: Path, timeout: int):
    # runner.rs caps child CPU at 2x the timeout and disables core dumps.
    inner = (
        f"ulimit -t {timeout * 2} 2>/dev/null; ulimit -c 0 2>/dev/null; "
        f"exec {shlex.quote(bin_path)} run {shlex.quote(str(fixture))}"
    )
    return harness_lib.run_fixture(
        ["/bin/sh", "-c", inner], timeout, text=False,
    )


def lossy(b) -> str:
    """utf-8 lossy decode; harness_lib's empty-fallback may already be str."""
    if isinstance(b, str):
        return b
    return bytes(b).decode("utf-8", errors="replace")


def first_diff(expected: str, actual: str) -> str:
    el, al = expected.splitlines(), actual.splitlines()
    for i, (e, a) in enumerate(zip(el, al)):
        if e != a:
            return f"line {i + 1}: oracle={e[:60]!r} mamba={a[:60]!r}"
    return f"line-count: oracle={len(el)} mamba={len(al)}"


def classify(rel: str, ctx) -> tuple[str, str, str]:
    """Return (rel, verdict, detail) with runner.rs::run_conformance parity."""
    path = FIXTURES_DIR / rel
    if "bench" in Path(rel).parts:
        return rel, SKIP, "bench (perf-pin owned)"
    try:
        src = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as err:
        return rel, INVALID, f"unreadable: {err}"
    xfail, strict_type = parse_directives(src)
    if any(line.lstrip().startswith("# RUN:") for line in src.splitlines()):
        return rel, SKIP, "pipeline fixture"
    if xfail is not None:
        return rel, XFAIL, xfail
    if strict_type or "type" in Path(rel).parts:
        return classify_type_strict(rel, path, ctx)

    cache_file = oracle_cache_path(ctx["salt"], src)
    try:
        expected = cache_file.read_bytes()
    except OSError:
        orc, oout, oerr = harness_lib.run_fixture(
            [ctx["python"], str(path)], ctx["timeout"], text=False,
        )
        if orc != 0:
            tail = lossy(oerr).strip().splitlines()
            return rel, INVALID, f"oracle rc={orc}: {tail[-1][:120] if tail else ''}"
        expected = oout if isinstance(oout, bytes) else lossy(oout).encode()
        oracle_cache_put(cache_file, expected)

    mrc, mout, merr = run_mamba(ctx["mamba"], path, ctx["timeout"])
    if mrc is None:
        return rel, FAIL, "<timeout>"
    if mrc != 0:
        tail = lossy(merr).strip().splitlines()
        return rel, FAIL, f"rc={mrc}: {tail[-1][:120] if tail else ''}"
    if mout == expected:
        return rel, PASS, ""
    return rel, DIVERGE, first_diff(lossy(expected), lossy(mout))


def classify_type_strict(rel: str, path: Path, ctx) -> tuple[str, str, str]:
    mrc, mout, merr = run_mamba(ctx["mamba"], path, ctx["timeout"])
    if mrc is None:
        return rel, FAIL, "<timeout>"
    out_s, err_s = lossy(mout), lossy(merr)
    if mrc != 0:
        if mrc < 0:
            return rel, FAIL, f"signal {-mrc}"
        if "TypeError" in err_s or "type error" in err_s \
                or "TypeError" in out_s or "type error" in out_s:
            return rel, PASS, "STRICT_TYPE_OK compile-time"
        tail = err_s.strip().splitlines()
        return rel, FAIL, (
            f"STRICT_TYPE_WRONG_EXCEPTION rc={mrc}: "
            f"{tail[-1][:120] if tail else ''}"
        )
    has_te = any(ln.startswith("typeerror:") for ln in out_s.splitlines())
    has_no = any(ln.startswith("no_typeerror:") for ln in out_s.splitlines())
    if has_te and not has_no:
        return rel, PASS, "STRICT_TYPE_OK runtime"
    if has_no and not has_te:
        return rel, FAIL, "MAMBA_TYPE_LEAKED"
    return rel, FAIL, "malformed type-strict output"


def all_fixtures() -> list[str]:
    out = []
    for p in sorted(FIXTURES_DIR.rglob("*.py")):
        relp = p.relative_to(FIXTURES_DIR)
        if any(part.startswith(".") for part in relp.parts):
            continue  # .cache and friends
        out.append(str(relp))
    return out


def load_state() -> list[str]:
    try:
        return [
            ln.strip() for ln in STATE_FILE.read_text().splitlines() if ln.strip()
        ]
    except OSError:
        return []


def select_fixtures(args) -> list[str]:
    selected: list[str] = []
    if args.all:
        selected = all_fixtures()
    elif args.failures:
        selected = load_state()
        if not selected:
            sys.exit(
                f"no stored failure set at {STATE_FILE}; "
                "run `sweep.py --all --store` first"
            )
    elif args.sample:
        failed = set(load_state())
        pool = [f for f in all_fixtures() if f not in failed]
        rng = random.Random(args.seed)
        selected = sorted(rng.sample(pool, min(args.sample, len(pool))))
    elif args.list:
        selected = [
            ln.strip()
            for ln in Path(args.list).read_text().splitlines()
            if ln.strip()
        ]
    if args.paths:
        for raw in args.paths:
            p = Path(raw)
            if not p.is_absolute():
                p = FIXTURES_DIR / raw
            if p.is_dir():
                selected.extend(
                    str(f.relative_to(FIXTURES_DIR)) for f in sorted(p.rglob("*.py"))
                )
            elif p.is_file():
                selected.append(str(p.relative_to(FIXTURES_DIR)))
            else:
                sys.exit(f"no such fixture or directory: {raw}")
    if args.filter:
        selected = [f for f in selected if args.filter in f]
    # de-dup, preserve order
    seen: set[str] = set()
    return [f for f in selected if not (f in seen or seen.add(f))]


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("paths", nargs="*", help="fixture files/dirs (relative to tests/cpython)")
    ap.add_argument("--all", action="store_true", help="sweep every fixture")
    ap.add_argument("--failures", action="store_true", help="re-run the stored failure set")
    ap.add_argument("--sample", type=int, metavar="N",
                    help="random canary of N fixtures outside the stored failure set")
    ap.add_argument("--seed", type=int, default=0, help="sample RNG seed (default 0)")
    ap.add_argument("--list", metavar="FILE", help="file with one fixture path per line")
    ap.add_argument("--filter", metavar="SUBSTR", help="keep only paths containing SUBSTR")
    ap.add_argument("--store", action="store_true",
                    help="merge results into the stored failure set")
    ap.add_argument("--out", metavar="FILE", help="also write this run's failures to FILE")
    ap.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
    ap.add_argument("--jobs", type=int, default=min(32, (os.cpu_count() or 4) * 2),
                    help="parallel workers (spawn-bound; default 2x cores)")
    ap.add_argument("--verbose", action="store_true", help="print PASS lines too")
    args = ap.parse_args(argv)

    selected = select_fixtures(args)
    if not selected:
        ap.error("nothing selected — pass paths, --list, --failures, --sample, or --all")

    python = python3_bin()
    ctx = {
        "mamba": mamba_bin(),
        "python": python,
        "salt": python_version_salt(python),
        "timeout": args.timeout,
    }
    print(
        f"sweep: {len(selected)} fixtures, {args.jobs} jobs, "
        f"bin={ctx['mamba']}, oracle-cache={ORACLE_CACHE_DIR}",
        file=sys.stderr,
    )

    counts: Counter[str] = Counter()
    failures: list[tuple[str, str, str]] = []
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for rel, verdict, detail in ex.map(lambda r: classify(r, ctx), selected):
            counts[verdict] += 1
            if verdict in BAD:
                failures.append((rel, verdict, detail))
                print(f"{verdict:8} {rel} — {detail}")
            elif args.verbose:
                print(f"{verdict:8} {rel}{' — ' + detail if detail else ''}")

    total = sum(counts.values())
    print(
        f"\n{total} swept: "
        + " ".join(f"{k}={counts[k]}" for k in (PASS, FAIL, DIVERGE, XFAIL, SKIP, INVALID) if counts[k])
    )

    failed_set = {rel for rel, _, _ in failures}
    if args.out:
        Path(args.out).write_text("\n".join(sorted(failed_set)) + ("\n" if failed_set else ""))
    if args.store:
        covered = set(selected)
        merged = sorted((set(load_state()) - covered) | failed_set)
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        STATE_FILE.write_text("\n".join(merged) + ("\n" if merged else ""))
        print(f"stored failure set: {len(merged)} entries -> {STATE_FILE}", file=sys.stderr)

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
