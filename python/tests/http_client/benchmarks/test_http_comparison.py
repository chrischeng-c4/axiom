"""
Head-to-head comparison: data-bridge HTTP client vs httpx.

Tests if data-bridge's Rust-based HTTP client is faster than httpx.
Uses cclab.probe benchmark utilities for timing and comparison.
"""
import asyncio
import httpx
from cclab.photon import HttpClient
from cclab.probe import (
    TestSuite, test, expect,
    benchmark, compare_benchmarks, BenchmarkGroup,
)
from tests.base import HttpTestSuite


# Test configuration
BASE_URL = "https://httpbin.org"
SAMPLE_JSON = {"name": "Alice", "age": 30, "city": "San Francisco"}


class TestHttpBenchmarks(HttpTestSuite):
    """HTTP client benchmark comparisons."""

    async def setup_suite(self):
        """Create HTTP clients for both frameworks."""
        self.db_client = HttpClient(base_url=BASE_URL, timeout=30.0)
        self.httpx_client = httpx.AsyncClient(base_url=BASE_URL, timeout=30.0)

    async def teardown_suite(self):
        """Clean up HTTP clients."""
        await self.httpx_client.aclose()

    # ===================
    # GET Requests
    # ===================

    @test(tags=["benchmark", "http", "get"])
    async def test_get_comparison(self):
        """Compare GET request performance: data-bridge vs httpx."""
        async def db_get():
            return await self.db_client.get("/get")

        async def httpx_get():
            return await self.httpx_client.get("/get")

        db_result = await benchmark(db_get, name="data-bridge", iterations=20, rounds=3)
        httpx_result = await benchmark(httpx_get, name="httpx", iterations=20, rounds=3)

        print("\n" + "=" * 60)
        print("GET Request Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        # Verify both succeeded
        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()

    # ===================
    # GET with Query Params
    # ===================

    @test(tags=["benchmark", "http", "get"])
    async def test_get_params_comparison(self):
        """Compare GET with params: data-bridge vs httpx."""
        async def db_get():
            return await self.db_client.get("/get", params={"foo": "bar", "count": "100"})

        async def httpx_get():
            return await self.httpx_client.get("/get", params={"foo": "bar", "count": "100"})

        db_result = await benchmark(db_get, name="data-bridge", iterations=20, rounds=3)
        httpx_result = await benchmark(httpx_get, name="httpx", iterations=20, rounds=3)

        print("\n" + "=" * 60)
        print("GET with Params Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()

    # ===================
    # POST JSON
    # ===================

    @test(tags=["benchmark", "http", "post"])
    async def test_post_json_comparison(self):
        """Compare POST JSON: data-bridge vs httpx."""
        async def db_post():
            return await self.db_client.post("/post", json=SAMPLE_JSON)

        async def httpx_post():
            return await self.httpx_client.post("/post", json=SAMPLE_JSON)

        db_result = await benchmark(db_post, name="data-bridge", iterations=20, rounds=3)
        httpx_result = await benchmark(httpx_post, name="httpx", iterations=20, rounds=3)

        print("\n" + "=" * 60)
        print("POST JSON Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()

    # ===================
    # Sequential Requests (Connection Reuse)
    # ===================

    @test(tags=["benchmark", "http", "sequential"])
    async def test_sequential_comparison(self):
        """Compare 10 sequential requests: data-bridge vs httpx."""
        async def db_sequential():
            results = []
            for i in range(10):
                response = await self.db_client.get("/get", params={"i": str(i)})
                results.append(response)
            return results

        async def httpx_sequential():
            results = []
            for i in range(10):
                response = await self.httpx_client.get("/get", params={"i": str(i)})
                results.append(response)
            return results

        db_result = await benchmark(db_sequential, name="data-bridge", iterations=5, rounds=3)
        httpx_result = await benchmark(httpx_sequential, name="httpx", iterations=5, rounds=3)

        print("\n" + "=" * 60)
        print("Sequential Requests (10x) Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()

    # ===================
    # Concurrent Requests
    # ===================

    @test(tags=["benchmark", "http", "concurrent"])
    async def test_concurrent_comparison(self):
        """Compare 10 concurrent requests: data-bridge vs httpx."""
        async def db_concurrent():
            tasks = [
                self.db_client.get("/get", params={"i": str(i)})
                for i in range(10)
            ]
            return await asyncio.gather(*tasks)

        async def httpx_concurrent():
            tasks = [
                self.httpx_client.get("/get", params={"i": str(i)})
                for i in range(10)
            ]
            return await asyncio.gather(*tasks)

        db_result = await benchmark(db_concurrent, name="data-bridge", iterations=5, rounds=3)
        httpx_result = await benchmark(httpx_concurrent, name="httpx", iterations=5, rounds=3)

        print("\n" + "=" * 60)
        print("Concurrent Requests (10x) Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()

    # ===================
    # Response Parsing
    # ===================

    @test(tags=["benchmark", "http", "json"])
    async def test_json_parse_comparison(self):
        """Compare GET + JSON parsing: data-bridge vs httpx."""
        async def db_parse():
            response = await self.db_client.get("/json")
            return response.json()

        async def httpx_parse():
            response = await self.httpx_client.get("/json")
            return response.json()

        db_result = await benchmark(db_parse, name="data-bridge", iterations=20, rounds=3)
        httpx_result = await benchmark(httpx_parse, name="httpx", iterations=20, rounds=3)

        print("\n" + "=" * 60)
        print("GET + JSON Parse Benchmark")
        print(compare_benchmarks([db_result, httpx_result], "data-bridge"))

        expect(db_result.success).to_be_true()
        expect(httpx_result.success).to_be_true()


# Run benchmarks when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([TestHttpBenchmarks], verbose=True)
