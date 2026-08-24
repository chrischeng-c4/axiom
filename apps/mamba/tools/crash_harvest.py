#!/usr/bin/env python3
"""Crash Report Harvester for Mamba integration tests (work-item #2604 H1).

Parses macOS DiagnosticReports (.ips) for cpython_ported_integration crash attribution.
"""

from __future__ import annotations

import argparse
import collections
import json
import os
import sys
from pathlib import Path


def extract_symbols(report: dict) -> list[str]:
    """Extract symbol names from faulting thread frames, outermost first."""
    ft = report.get("faultingThread", 0)
    threads = report.get("threads", [])
    symbols: list[str] = []
    if isinstance(threads, list) and isinstance(ft, int) and 0 <= ft < len(threads):
        thread = threads[ft]
        if isinstance(thread, dict):
            frames = thread.get("frames", [])
            if isinstance(frames, list):
                for f in frames:
                    if isinstance(f, dict):
                        sym = f.get("symbol")
                        if sym and isinstance(sym, str):
                            symbols.append(sym)
                        else:
                            symbols.append("")
    return symbols


def classify_family(report: dict) -> str:
    """Classify crash report into crash family according to operational precedence rule."""
    asi_text = json.dumps(report.get("asi", {}))
    symbols = extract_symbols(report)

    if "memory corruption of free block" in asi_text:
        return "FREE_BLOCK_TRAP"
    if any("debug_validate_obj" in s for s in symbols):
        return "DETECTOR_ABORT"
    if any("panic_nounwind" in s or "panic_cannot_unwind" in s for s in symbols):
        return "DETECTOR_ABORT"

    exc = report.get("exception", {})
    sig = ""
    if isinstance(exc, dict):
        sig = exc.get("signal", "") or ""

    if sig == "SIGABRT":
        return "MALLOC_ABORT"

    return f"OTHER_{sig}"


def extract_top_mamba(report: dict) -> str:
    """Extract top mamba frame symbol without ::h<hex> symbol-hash suffix."""
    symbols = extract_symbols(report)
    for s in symbols:
        if s.startswith("mamba::"):
            return s.split("::h")[0]
    return "(no mamba frame)"


def process_corpus(
    corpus_dir: Path,
) -> tuple[int, int, dict[str, int], dict[tuple[str, str], int]]:
    """Process all cpython_ported_integration-*.ips files in corpus_dir."""
    files = sorted(corpus_dir.glob("cpython_ported_integration-*.ips"))

    families_count: dict[str, int] = collections.Counter()
    pairs_count: dict[tuple[str, str], int] = collections.Counter()
    total_reports = 0
    skipped_reports = 0

    for path in files:
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
            parts = text.split("\n", 1)
            if len(parts) < 2:
                skipped_reports += 1
                continue
            report = json.loads(parts[1])
        except Exception:
            skipped_reports += 1
            continue

        total_reports += 1
        fam = classify_family(report)
        tm = extract_top_mamba(report)

        families_count[fam] += 1
        pairs_count[(fam, tm)] += 1

    return total_reports, skipped_reports, families_count, pairs_count


def print_summary(
    total_reports: int,
    skipped_reports: int,
    families_count: dict[str, int],
    pairs_count: dict[tuple[str, str], int],
) -> None:
    """Print deterministic summary to stdout according to CLI contract."""
    print(f"TOTAL REPORTS: {total_reports}")
    print(f"SKIPPED: {skipped_reports}\n")

    print("BY FAMILY:")
    for fam in sorted(families_count.keys()):
        print(f"  {families_count[fam]}  {fam}")
    print()

    print("BY (FAMILY, TOP MAMBA FRAME):")
    for fam, frame in sorted(pairs_count.keys()):
        print(f"  {pairs_count[(fam, frame)]}  {fam}  {frame}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Harvester for cpython_ported_integration crash reports"
    )
    parser.add_argument(
        "--corpus-dir",
        type=str,
        default="~/Library/Logs/DiagnosticReports",
        help="Directory containing cpython_ported_integration-*.ips files",
    )

    args = parser.parse_args()
    corpus_dir = Path(os.path.expanduser(args.corpus_dir)).resolve()

    total_reports, skipped_reports, families_count, pairs_count = process_corpus(corpus_dir)
    print_summary(total_reports, skipped_reports, families_count, pairs_count)
    if skipped_reports > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
