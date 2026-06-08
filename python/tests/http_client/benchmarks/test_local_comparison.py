"""
HTTP Client Benchmark: data-bridge vs httpx

Declarative benchmark using the new simplified API.
~40 lines vs ~300 lines before.
"""

import httpx
from cclab.photon import HttpClient
from cclab.probe import HttpBenchmark


# Declarative routes (server responds with these JSON payloads)
ROUTES = {
    "/get": {"status": "ok", "method": "GET"},
    "/params": {"foo": "bar", "count": 100},
    "/json": {"name": "Alice", "age": 30, "items": list(range(100))},
    "/large": {"data": list(range(1000)), "nested": [{"name": "Alice"}] * 10},
}


# Create benchmark with declarative configuration
bench = HttpBenchmark(
    name="HTTP Client Performance: data-bridge vs httpx",
    description="Local server benchmarks with enhanced statistics (percentiles, outliers, CI)",
    routes=ROUTES,
    clients={
        "data-bridge": lambda url: HttpClient(base_url=url, timeout=10.0),
        "httpx": lambda url: httpx.AsyncClient(base_url=url, timeout=10.0),
    },
    baseline="data-bridge",
)


if __name__ == "__main__":
    bench.run()
