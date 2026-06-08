"""
HTTP Benchmark - Declarative HTTP client benchmarking

Provides a simple, declarative API for benchmarking HTTP clients
against a built-in test server.

Example:
    from cclab.qc import HttpBenchmark
    from cclab.http import HttpClient
    import httpx

    bench = HttpBenchmark(
        name="HTTP Client Comparison",
        routes={
            "/get": {"status": "ok"},
            "/json": {"data": list(range(100))},
        },
        clients={
            "data-bridge": lambda url: HttpClient(base_url=url),
            "httpx": lambda url: httpx.AsyncClient(base_url=url),
        },
        baseline="data-bridge",
    )

    if __name__ == "__main__":
        bench.run()
"""

from __future__ import annotations

import asyncio
import platform
import sys
from typing import Any, Callable, Dict, List, Optional, Union

from .. import cclab as _rust_module

_test = _rust_module.qc

# Import Rust types
TestServer = _test.TestServer
TestServerHandle = _test.TestServerHandle
BenchmarkReport = _test.BenchmarkReport
BenchmarkReportGroup = _test.BenchmarkReportGroup
BenchmarkEnvironment = _test.BenchmarkEnvironment
print_comparison_table = _test.print_comparison_table

from .benchmark import benchmark, BenchmarkResult


class HttpBenchmark:
    """
    Declarative HTTP client benchmark framework.

    Creates a test server with specified routes and benchmarks
    multiple HTTP clients against it.

    Example:
        bench = HttpBenchmark(
            name="HTTP Comparison",
            routes={"/get": {"status": "ok"}},
            clients={
                "data-bridge": lambda url: HttpClient(base_url=url),
                "httpx": lambda url: httpx.AsyncClient(base_url=url),
            },
        )
        bench.run()
    """

    def __init__(
        self,
        name: str,
        routes: Dict[str, Any],
        clients: Dict[str, Callable[[str], Any]],
        *,
        baseline: Optional[str] = None,
        description: Optional[str] = None,
        port: int = 0,
    ):
        """
        Create an HTTP benchmark.

        Args:
            name: Benchmark name/title.
            routes: Dict mapping paths to JSON responses.
                    e.g., {"/get": {"status": "ok"}}
            clients: Dict mapping client names to factory functions.
                     Factory takes base_url and returns async HTTP client.
            baseline: Name of baseline client for comparison (default: first).
            description: Optional description for reports.
            port: Port for test server (0 = auto-select).
        """
        self.name = name
        self.routes = routes
        self.clients = clients
        self.baseline = baseline or next(iter(clients.keys()), None)
        self.description = description
        self.port = port

        # Store results
        self._results: List[BenchmarkResult] = []
        self._report: Optional[BenchmarkReport] = None

    def run(
        self,
        *,
        auto: bool = True,
        rounds: int = 5,
        warmup: int = 10,
        endpoints: Optional[List[str]] = None,
        save_reports: bool = True,
        report_prefix: str = "benchmark_report",
    ) -> BenchmarkReport:
        """
        Run the benchmark suite.

        Args:
            auto: Auto-calibrate iterations (recommended).
            rounds: Number of benchmark rounds.
            warmup: Number of warmup iterations.
            endpoints: List of endpoints to test (default: all routes).
            save_reports: Save HTML/JSON/MD reports.
            report_prefix: Prefix for report filenames.

        Returns:
            BenchmarkReport with all results.
        """
        return asyncio.run(
            self._run_async(
                auto=auto,
                rounds=rounds,
                warmup=warmup,
                endpoints=endpoints,
                save_reports=save_reports,
                report_prefix=report_prefix,
            )
        )

    async def _run_async(
        self,
        *,
        auto: bool,
        rounds: int,
        warmup: int,
        endpoints: Optional[List[str]],
        save_reports: bool,
        report_prefix: str,
    ) -> BenchmarkReport:
        """Internal async benchmark runner."""

        # Create and start test server
        server = TestServer()
        server.port(self.port if self.port > 0 else 0)

        for path, response in self.routes.items():
            server.get(path, response)

        print(f"\nStarting test server...")
        handle = await server.start()
        base_url = handle.url
        print(f"Server running at {base_url}")

        try:
            # Create report
            report = BenchmarkReport(
                self.name,
                self.description or f"HTTP benchmark comparing {', '.join(self.clients.keys())}"
            )

            env = BenchmarkEnvironment(
                python_version=sys.version.split()[0],
                platform=platform.platform(),
                cpu=platform.processor() or "unknown",
                hostname=platform.node(),
            )
            report.set_environment(env)

            # Determine endpoints
            test_endpoints = endpoints or list(self.routes.keys())

            # Create clients
            client_instances = {}
            for name, factory in self.clients.items():
                client_instances[name] = factory(base_url)

            # Benchmark each endpoint
            for i, endpoint in enumerate(test_endpoints, 1):
                print(f"\n[{i}/{len(test_endpoints)}] Benchmarking {endpoint}...")

                group = BenchmarkReportGroup(f"GET {endpoint}", self.baseline)
                group_results = []

                for client_name, client in client_instances.items():
                    # Create benchmark function
                    async def bench_func(c=client, ep=endpoint):
                        return await c.get(ep)

                    result = await benchmark(
                        bench_func,
                        name=client_name,
                        auto=auto,
                        rounds=rounds,
                        warmup=warmup,
                    )

                    # Auto-print detailed stats
                    result.print_detailed()

                    group.add_result(result)
                    group_results.append(result)

                # Print comparison for this endpoint
                print_comparison_table(group_results, self.baseline)

                report.add_group(group)

            # Cleanup clients
            for name, client in client_instances.items():
                if hasattr(client, "aclose"):
                    await client.aclose()
                elif hasattr(client, "close"):
                    client.close()

            # Print summary
            print("\n" + "=" * 82)
            print("SUMMARY - All Benchmarks")
            print("=" * 82)

            for group in report.groups:
                print(f"\n{group.name}:")
                print_comparison_table(group.results, self.baseline)

            # Save reports
            if save_reports:
                report.save(f"{report_prefix}.html", "html")
                report.save(f"{report_prefix}.json", "json")
                report.save(f"{report_prefix}.md", "markdown")
                print("\n" + "=" * 82)
                print("Reports saved:")
                print(f"  - {report_prefix}.html (interactive charts)")
                print(f"  - {report_prefix}.json (raw data)")
                print(f"  - {report_prefix}.md   (markdown)")
                print("=" * 82)

            self._report = report
            return report

        finally:
            # Stop server
            handle.stop()
            print("\nServer stopped.")
