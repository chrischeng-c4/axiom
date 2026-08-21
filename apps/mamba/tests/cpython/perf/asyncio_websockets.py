# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "asyncio_websockets"
# dimension = "perf"
# case = "asyncio_websockets"
# subject = "pyperformance asyncio_websockets"
# kind = "bench"
# xfail = "mamba must run the pyperformance asyncio_websockets workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_asyncio_websockets/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance asyncio_websockets workload faster than CPython on CPU+RSS
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
Benchmark for asyncio websocket server and client performance
transferring 1MB of data.

Author: Kumar Aditya
"""

import pyperf
import websockets.server
import websockets.client
import websockets.exceptions
import asyncio

CHUNK_SIZE = 1024 ** 2
DATA = b"x" * CHUNK_SIZE

stop: asyncio.Event


async def handler(websocket) -> None:
    for _ in range(100):
        await websocket.recv()

    stop.set()


async def send(ws):
    try:
        await ws.send(DATA)
    except websockets.exceptions.ConnectionClosedOK:
        pass


async def main() -> None:
    global stop
    t0 = pyperf.perf_counter()
    stop = asyncio.Event()
    async with websockets.server.serve(handler, "", 8001):
        async with websockets.client.connect("ws://localhost:8001") as ws:
            await asyncio.gather(*[send(ws) for _ in range(100)])
        await stop.wait()
    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = "Benchmark asyncio websockets"
    runner.bench_async_func('asyncio_websockets', main)
