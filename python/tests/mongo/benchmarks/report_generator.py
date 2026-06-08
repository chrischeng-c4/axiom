"""
Generate benchmark reports from pytest-benchmark JSON output.

Analyzes benchmark results and generates comparison tables showing
relative performance of data-bridge vs other MongoDB frameworks.
"""

import json
from pathlib import Path
from typing import Dict, Any, List
from datetime import datetime
from tabulate import tabulate


class BenchmarkReportGenerator:
    """Generate human-readable reports from benchmark results."""

    def __init__(self, results_file: Path):
        """
        Initialize report generator.

        Args:
            results_file: Path to pytest-benchmark JSON output file
        """
        with open(results_file) as f:
            self.data = json.load(f)

        # Extract benchmark results
        self.benchmarks = self.data.get("benchmarks", [])

        # Group benchmarks by operation and framework
        self.grouped = self._group_benchmarks()

    def _group_benchmarks(self) -> Dict[str, Dict[str, Any]]:
        """
        Group benchmarks by operation type and framework.

        Returns:
            Dictionary: {operation: {framework: {batch_size: stats}}}
        """
        grouped = {}

        for bench in self.benchmarks:
            # Parse test name: test_{framework}_{operation}[batch_size]
            # Example: test_cclab_insert_many[10]
            name = bench["name"]
            group = bench.get("group", "unknown")

            # Extract framework and batch size
            parts = name.split("[")
            test_name = parts[0]
            batch_size = parts[1].rstrip("]") if len(parts) > 1 else "single"

            # Extract framework from test name
            framework = self._extract_framework(test_name)

            # Store stats
            if group not in grouped:
                grouped[group] = {}
            if framework not in grouped[group]:
                grouped[group][framework] = {}

            grouped[group][framework][batch_size] = bench["stats"]

        return grouped

    def _extract_framework(self, test_name: str) -> str:
        """Extract framework name from test function name."""
        if "cclab" in test_name:
            return "cclab"
        elif "beanie" in test_name:
            return "beanie"
        elif "motor" in test_name:
            return "motor"
        elif "pymongo_sync" in test_name:
            return "pymongo_sync"
        elif "pymongo_gevent" in test_name:
            return "pymongo_gevent"
        elif "mongoengine" in test_name:
            return "mongoengine"
        return "unknown"

    def generate_comparison_table(
        self,
        operation: str,
        baseline: str = "cclab",
        stat: str = "mean"
    ) -> str:
        """
        Generate markdown comparison table for an operation.

        Args:
            operation: Operation group name (e.g., "bulk-insert")
            baseline: Framework to use as baseline (default: cclab)
            stat: Statistic to compare (mean, median, min, max, stddev)

        Returns:
            Markdown formatted table
        """
        if operation not in self.grouped:
            return f"No data for operation: {operation}"

        op_data = self.grouped[operation]
        frameworks = ["cclab", "beanie", "motor", "pymongo_sync", "pymongo_gevent", "mongoengine"]

        # Get all batch sizes
        batch_sizes = set()
        for fw_data in op_data.values():
            batch_sizes.update(fw_data.keys())
        batch_sizes = sorted(batch_sizes, key=lambda x: int(x) if x.isdigit() else 0)

        # Build table
        headers = ["Batch Size", "data-bridge (s)", "Beanie", "motor", "pymongo", "gevent", "MongoEngine"]
        rows = []

        for batch_size in batch_sizes:
            row = [batch_size]

            # Get baseline time
            baseline_time = None
            if baseline in op_data and batch_size in op_data[baseline]:
                baseline_time = op_data[baseline][batch_size][stat]
                row.append(f"{baseline_time:.6f}")
            else:
                row.append("N/A")
                continue  # Skip if no baseline

            # Compare other frameworks
            for fw in frameworks[1:]:  # Skip cclab (already added)
                if fw in op_data and batch_size in op_data[fw]:
                    fw_time = op_data[fw][batch_size][stat]
                    speedup = fw_time / baseline_time
                    row.append(f"{speedup:.2f}x slower" if speedup > 1 else f"{1/speedup:.2f}x faster")
                else:
                    row.append("N/A")

            rows.append(row)

        return tabulate(rows, headers=headers, tablefmt="github")

    def generate_full_report(self, output_file: Path = None) -> str:
        """
        Generate complete benchmark report in markdown.

        Args:
            output_file: Optional path to save the report

        Returns:
            Markdown formatted report
        """
        report_lines = [
            "# MongoDB ORM Benchmark Report",
            "",
            f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            "",
            "## Executive Summary",
            "",
            "This report compares the performance of data-bridge (Rust backend) against",
            "other Python MongoDB libraries across various operations and batch sizes.",
            "",
            "## Test Environment",
            "",
            f"- **Python**: {self.data.get('machine_info', {}).get('python_version', 'N/A')}",
            f"- **Platform**: {self.data.get('machine_info', {}).get('platform', 'N/A')}",
            f"- **CPU**: {self.data.get('machine_info', {}).get('cpu', {}).get('brand_raw', 'N/A')}",
            "",
            "## Frameworks Compared",
            "",
            "1. **data-bridge**: Rust-backed MongoDB ORM with zero Python byte handling",
            "2. **Beanie**: Async ODM built on motor/pymongo-async",
            "3. **motor**: Pure pymongo-async (no ODM layer)",
            "4. **pymongo**: Standard synchronous pymongo",
            "5. **gevent + pymongo**: Greenlet-based concurrency",
            "6. **MongoEngine**: Traditional sync ODM",
            "",
        ]

        # Generate tables for each operation
        operations = {
            "bulk-insert": "Bulk Insert Performance",
            "single-insert": "Single Insert Performance",
            "find-one": "find_one Query Performance",
            "find-many": "find_many Query Performance",
            "count": "count() Performance",
            "update-single": "Single Update Performance",
            "update-bulk": "Bulk Update Performance",
            "delete-single": "Single Delete Performance",
            "delete-bulk": "Bulk Delete Performance",
        }

        for op_key, op_title in operations.items():
            if op_key in self.grouped:
                report_lines.extend([
                    f"## {op_title}",
                    "",
                    self.generate_comparison_table(op_key),
                    "",
                ])

        # Generate conclusions
        report_lines.extend([
            "## Interpretation Guide",
            "",
            "- **Baseline**: data-bridge performance is shown in seconds",
            "- **Comparisons**: `2.5x slower` means the framework took 2.5x longer than data-bridge",
            "- **Faster is better**: Lower numbers = better performance",
            "",
            "## Key Findings",
            "",
            self._generate_key_findings(),
            "",
        ])

        report = "\n".join(report_lines)

        if output_file:
            output_file.write_text(report)

        return report

    def _generate_key_findings(self) -> str:
        """
        Analyze results and generate key findings.

        Returns:
            Markdown formatted findings
        """
        findings = []

        # Bulk insert analysis
        if "bulk-insert" in self.grouped:
            bulk_data = self.grouped["bulk-insert"]
            if "cclab" in bulk_data and "beanie" in bulk_data:
                # Compare 1000-doc bulk insert
                if "1000" in bulk_data["cclab"] and "1000" in bulk_data["beanie"]:
                    db_time = bulk_data["cclab"]["1000"]["mean"]
                    beanie_time = bulk_data["beanie"]["1000"]["mean"]
                    speedup = beanie_time / db_time
                    findings.append(f"- **Bulk Insert (1000 docs)**: data-bridge is {speedup:.1f}x faster than Beanie")

        # Find one analysis
        if "find-one" in self.grouped:
            find_data = self.grouped["find-one"]
            if "cclab" in find_data and "motor" in find_data:
                if "single" in find_data["cclab"] and "single" in find_data["motor"]:
                    db_time = find_data["cclab"]["single"]["mean"]
                    motor_time = find_data["motor"]["single"]["mean"]
                    speedup = motor_time / db_time
                    findings.append(f"- **find_one**: data-bridge is {speedup:.1f}x {'faster' if speedup > 1 else 'slower'} than raw motor")

        if not findings:
            findings.append("- Analysis pending: Run benchmarks to generate findings")

        return "\n".join(findings)


def main():
    """CLI entry point for report generation."""
    import sys
    from argparse import ArgumentParser

    parser = ArgumentParser(description="Generate benchmark report from pytest-benchmark JSON output")
    parser.add_argument("results_file", type=Path, help="Path to benchmark JSON results")
    parser.add_argument("-o", "--output", type=Path, help="Output markdown file path")

    args = parser.parse_args()

    if not args.results_file.exists():
        print(f"Error: Results file not found: {args.results_file}", file=sys.stderr)
        sys.exit(1)

    generator = BenchmarkReportGenerator(args.results_file)
    report = generator.generate_full_report(args.output)

    if args.output:
        print(f"Report saved to: {args.output}")
    else:
        print(report)


if __name__ == "__main__":
    main()
