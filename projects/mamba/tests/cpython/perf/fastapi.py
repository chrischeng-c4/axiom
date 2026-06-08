# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "fastapi"
# dimension = "perf"
# case = "fastapi"
# subject = "pyperformance fastapi"
# kind = "bench"
# xfail = "mamba must run the pyperformance fastapi workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_fastapi/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance fastapi workload faster than CPython on CPU+RSS
import sys as _sys, types as _t
class _Args:
    """Minimal argparser stand-in (no `import argparse`, which a sibling
    perf/argparse.py fixture would shadow). Records add_argument defaults."""
    def __init__(self):
        self._defaults = {}
    def add_argument(self, *names, **k):
        dest = k.get("dest")
        if not dest:
            for n in names:
                if isinstance(n, str) and n.startswith("--"):
                    dest = n[2:].replace("-", "_"); break
                if isinstance(n, str) and not n.startswith("-"):
                    dest = n; break
        if dest:
            self._defaults[dest] = k.get("default")
    def add_mutually_exclusive_group(self, *a, **k):
        return self
    def add_argument_group(self, *a, **k):
        return self
class _Runner:
    def __init__(self, *a, **k):
        self.metadata = {}
        self.argparser = _Args()
    def parse_args(self, *a, **k):
        return _t.SimpleNamespace(**self.argparser._defaults)
    def bench_func(self, name, func, *args, **k):
        func(*args)                       # func runs the workload itself
    def bench_time_func(self, name, func, *args, **k):
        func(1, *args)                    # pyperf passes loops as the 1st arg
    def bench_async_func(self, name, func, *args, **k):
        import asyncio
        asyncio.run(func(*args))
def _reg(_name, _code):
    _m = _t.ModuleType(_name)
    exec(compile(_code, _name, "exec"), _m.__dict__)
    _sys.modules[_name] = _m
_p = _t.ModuleType("pyperf")
_p.Runner = _Runner
def _pc():
    import time
    return time.perf_counter()
_p.perf_counter = _pc
_sys.modules["pyperf"] = _p

"""
Test the performance of simple HTTP serving with FastAPI.

This benchmark tests FastAPI's request handling, including:
- Path parameter extraction and validation
- Pydantic model serialization
- JSON response encoding

The bench serves a REST API endpoint that returns JSON objects,
simulating a typical web application scenario.

Author: Savannah Ostrowski
"""

import asyncio
import socket

import httpx
import pyperf
import threading
import uvicorn
from fastapi import FastAPI
from pydantic import BaseModel

HOST = "127.0.0.1"

CONCURRENCY = 150

class Item(BaseModel):
    id: int
    name: str
    price: float
    tags: list[str] = []

app = FastAPI()

@app.get("/items/{item_id}", response_model=Item)
async def get_item(item_id: int):
    return {
        "id": item_id,
        "name": "Sample Item",
        "price": 9.99,
        "tags": ["sample", "item", "fastapi"]
    }

def setup_server(): 
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((HOST, 0))
        s.listen(1)
        port = s.getsockname()[1]

    config = uvicorn.Config(app, host=HOST, port=port, log_level="error")
    server = uvicorn.Server(config)

    server_thread = threading.Thread(target=server.run, daemon=True)
    server_thread.start()

    while not server.started:
        pass

    url = f"http://{HOST}:{port}"
    return url

def bench_fastapi(loops, url):
    async def run_benchmark():
        async with httpx.AsyncClient() as client:
            t0 = pyperf.perf_counter()

            for i in range(loops):
                tasks = [
                    client.get(f"{url}/items/{i}")
                    for _ in range(CONCURRENCY)
                ]
                responses = await asyncio.gather(*tasks)
                for response in responses:
                    response.raise_for_status()
                    data = response.json()
                    assert data["id"] == i
                    assert "tags" in data

            return pyperf.perf_counter() - t0

    return asyncio.run(run_benchmark())


if __name__ == "__main__":
    url = setup_server()
    runner = pyperf.Runner()
    runner.metadata['description'] = "Test the performance of HTTP requests with FastAPI"
    runner.bench_time_func("fastapi_http", bench_fastapi, url)
