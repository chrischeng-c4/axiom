"""Entry point: python -m benchmarks.orbit

Discovers and runs all orbit benchmark files, producing a comparison
report with orbit as the baseline.
"""

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

from cclab.probe import discover_benchmarks, run_benchmarks

from ._helpers import BACKENDS, has_uvloop

BENCHMARK_DIR = Path(__file__).parent
REPORTS_DIR = BENCHMARK_DIR / "reports"


def _print_header() -> None:
    print("=" * 70)
    print("Orbit vs uvloop vs asyncio  -  Event-Loop Benchmark Suite")
    print("=" * 70)
    print(f"\nBackends: {', '.join(BACKENDS)}")
    if not has_uvloop():
        print("  (uvloop not installed - skipping uvloop benchmarks)")
    print()


async def main() -> int:
    _print_header()

    info = discover_benchmarks(BENCHMARK_DIR)
    print(f"Discovered {len(info['files'])} file(s), {info['groups']} group(s)\n")

    if info["groups"] == 0:
        print("No benchmark groups found.")
        return 1

    report = await run_benchmarks(
        auto=True,
        rounds=5,
        baseline_name="orbit",
        title="Orbit vs uvloop vs asyncio",
        description="Event-loop micro-benchmarks comparing Orbit (Rust/Tokio), "
        "uvloop, and the standard asyncio loop.",
    )

    print("\n" + report.to_console())

    REPORTS_DIR.mkdir(exist_ok=True)
    report_path = str(REPORTS_DIR / "orbit_benchmark_report")
    report.save(report_path, "markdown")
    report.save(report_path, "json")
    print(f"\nReports saved: {report_path}.md, {report_path}.json")

    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
