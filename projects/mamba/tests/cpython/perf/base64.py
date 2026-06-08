# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "base64"
# dimension = "perf"
# case = "base64"
# subject = "pyperformance base64"
# kind = "bench"
# xfail = "mamba must run the pyperformance base64 workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_base64/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance base64 workload faster than CPython on CPU+RSS
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

"""Benchmark for the base64 module's primary public APIs.

Tests encoding and decoding performance across various variants
and data sizes, split into _small (balanced small data) and _large variants.

Small weighs towards measuring overhead, large measures the core algorithm
loop implementation.
"""

import base64
import random
import pyperf


# Generate test data with a fixed seed for reproducibility
random.seed(12345)
DATA_TINY = random.randbytes(20)
DATA_SMALL = random.randbytes(127)  # odd on purpose
DATA_MEDIUM = random.randbytes(3072)
DATA_9K = random.randbytes(9000)
DATA_LARGE = random.randbytes(102400)
DATA_HUGE = random.randbytes(1048574)  # 1M-2

# Pre-encoded data for decode benchmarks
B64_TINY = base64.b64encode(DATA_TINY)
B64_SMALL = base64.b64encode(DATA_SMALL)
B64_MEDIUM = base64.b64encode(DATA_MEDIUM)
B64_9K_STR = base64.b64encode(DATA_9K).decode('ascii')
B64_LARGE = base64.b64encode(DATA_LARGE)
B64_HUGE = base64.b64encode(DATA_HUGE)

B64_URLSAFE_TINY = base64.urlsafe_b64encode(DATA_TINY)
B64_URLSAFE_SMALL = base64.urlsafe_b64encode(DATA_SMALL)
B64_URLSAFE_MEDIUM = base64.urlsafe_b64encode(DATA_MEDIUM)
B64_URLSAFE_9K_STR = base64.urlsafe_b64encode(DATA_9K).decode('ascii')

B32_TINY = base64.b32encode(DATA_TINY)
B32_SMALL = base64.b32encode(DATA_SMALL)
B32_MEDIUM = base64.b32encode(DATA_MEDIUM)
B32_9K_STR = base64.b32encode(DATA_9K).decode('ascii')
B32_LARGE = base64.b32encode(DATA_LARGE)
B32_HUGE = base64.b32encode(DATA_HUGE)

B16_TINY = base64.b16encode(DATA_TINY)
B16_SMALL = base64.b16encode(DATA_SMALL)
B16_MEDIUM = base64.b16encode(DATA_MEDIUM)
B16_9K_STR = base64.b16encode(DATA_9K).decode('ascii')
B16_LARGE = base64.b16encode(DATA_LARGE)
B16_HUGE = base64.b16encode(DATA_HUGE)

A85_TINY = base64.a85encode(DATA_TINY)
A85_SMALL = base64.a85encode(DATA_SMALL)
A85_MEDIUM = base64.a85encode(DATA_MEDIUM)
A85_9K_STR = base64.a85encode(DATA_9K).decode('ascii')
A85_LARGE = base64.a85encode(DATA_LARGE)
A85_HUGE = base64.a85encode(DATA_HUGE)

B85_TINY = base64.b85encode(DATA_TINY)
B85_SMALL = base64.b85encode(DATA_SMALL)
B85_MEDIUM = base64.b85encode(DATA_MEDIUM)
B85_9K_STR = base64.b85encode(DATA_9K).decode('ascii')
B85_LARGE = base64.b85encode(DATA_LARGE)
B85_HUGE = base64.b85encode(DATA_HUGE)


# --- Base64 ---

def bench_b64_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.b64encode(DATA_TINY)
            base64.b64decode(B64_TINY)
        for _ in range(71):
            base64.b64encode(DATA_SMALL)
            base64.b64decode(B64_SMALL)
        for _ in range(3):
            base64.b64encode(DATA_MEDIUM)
            base64.b64decode(B64_MEDIUM)
        base64.b64encode(DATA_9K)
        base64.b64decode(B64_9K_STR)
    return pyperf.perf_counter() - t0


def bench_b64_large(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(10):
            base64.b64encode(DATA_LARGE)
            base64.b64decode(B64_LARGE)
        base64.b64encode(DATA_HUGE)
        base64.b64decode(B64_HUGE)
    return pyperf.perf_counter() - t0


# --- URL-safe Base64 (small only) ---

def bench_urlsafe_b64_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.urlsafe_b64encode(DATA_TINY)
            base64.urlsafe_b64decode(B64_URLSAFE_TINY)
        for _ in range(71):
            base64.urlsafe_b64encode(DATA_SMALL)
            base64.urlsafe_b64decode(B64_URLSAFE_SMALL)
        for _ in range(3):
            base64.urlsafe_b64encode(DATA_MEDIUM)
            base64.urlsafe_b64decode(B64_URLSAFE_MEDIUM)
        base64.urlsafe_b64encode(DATA_9K)
        base64.urlsafe_b64decode(B64_URLSAFE_9K_STR)
    return pyperf.perf_counter() - t0


# --- Base32 ---

def bench_b32_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.b32encode(DATA_TINY)
            base64.b32decode(B32_TINY)
        for _ in range(71):
            base64.b32encode(DATA_SMALL)
            base64.b32decode(B32_SMALL)
        for _ in range(3):
            base64.b32encode(DATA_MEDIUM)
            base64.b32decode(B32_MEDIUM)
        base64.b32encode(DATA_9K)
        base64.b32decode(B32_9K_STR)
    return pyperf.perf_counter() - t0


def bench_b32_large(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(10):
            base64.b32encode(DATA_LARGE)
            base64.b32decode(B32_LARGE)
        base64.b32encode(DATA_HUGE)
        base64.b32decode(B32_HUGE)
    return pyperf.perf_counter() - t0


# --- Base16 ---

def bench_b16_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.b16encode(DATA_TINY)
            base64.b16decode(B16_TINY)
        for _ in range(71):
            base64.b16encode(DATA_SMALL)
            base64.b16decode(B16_SMALL)
        for _ in range(3):
            base64.b16encode(DATA_MEDIUM)
            base64.b16decode(B16_MEDIUM)
        base64.b16encode(DATA_9K)
        base64.b16decode(B16_9K_STR)
    return pyperf.perf_counter() - t0


def bench_b16_large(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(10):
            base64.b16encode(DATA_LARGE)
            base64.b16decode(B16_LARGE)
        base64.b16encode(DATA_HUGE)
        base64.b16decode(B16_HUGE)
    return pyperf.perf_counter() - t0


# --- Ascii85 (includes wrapcol=76) ---

def bench_a85_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.a85encode(DATA_TINY)
            base64.a85encode(DATA_TINY, wrapcol=76)
            base64.a85decode(A85_TINY)
            base64.a85decode(A85_TINY)  # balance enc+dec weight
        for _ in range(71):
            base64.a85encode(DATA_SMALL)
            base64.a85encode(DATA_SMALL, wrapcol=76)
            base64.a85decode(A85_SMALL)
            base64.a85decode(A85_SMALL)  # balance enc+dec weight
        for _ in range(3):
            base64.a85encode(DATA_MEDIUM)
            base64.a85encode(DATA_MEDIUM, wrapcol=76)
            base64.a85decode(A85_MEDIUM)
            base64.a85decode(A85_MEDIUM)  # balance enc+dec weight
        base64.a85encode(DATA_9K)
        base64.a85encode(DATA_9K, wrapcol=76)
        base64.a85decode(A85_9K_STR)
        base64.a85decode(A85_9K_STR)  # balance enc+dec weight
    return pyperf.perf_counter() - t0


def bench_a85_large(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(10):
            base64.a85encode(DATA_LARGE)
            base64.a85encode(DATA_LARGE, wrapcol=76)
            base64.a85decode(A85_LARGE)
            base64.a85decode(A85_LARGE)  # balance enc+dec weight
        base64.a85encode(DATA_HUGE)
        base64.a85encode(DATA_HUGE, wrapcol=76)
        base64.a85decode(A85_HUGE)
        base64.a85decode(A85_HUGE)  # balance enc+dec weight
    return pyperf.perf_counter() - t0


# --- Base85 ---

def bench_b85_small(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(450):
            base64.b85encode(DATA_TINY)
            base64.b85decode(B85_TINY)
        for _ in range(71):
            base64.b85encode(DATA_SMALL)
            base64.b85decode(B85_SMALL)
        for _ in range(3):
            base64.b85encode(DATA_MEDIUM)
            base64.b85decode(B85_MEDIUM)
        base64.b85encode(DATA_9K)
        base64.b85decode(B85_9K_STR)
    return pyperf.perf_counter() - t0


def bench_b85_large(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()
    for _ in range_it:
        for _ in range(10):
            base64.b85encode(DATA_LARGE)
            base64.b85decode(B85_LARGE)
        base64.b85encode(DATA_HUGE)
        base64.b85decode(B85_HUGE)
    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = "Benchmark base64 module encoding/decoding"

    runner.bench_time_func('base64_small', bench_b64_small)
    runner.bench_time_func('base64_large', bench_b64_large)

    runner.bench_time_func('urlsafe_base64_small', bench_urlsafe_b64_small)

    runner.bench_time_func('base32_small', bench_b32_small)
    runner.bench_time_func('base32_large', bench_b32_large)

    runner.bench_time_func('base16_small', bench_b16_small)
    runner.bench_time_func('base16_large', bench_b16_large)

    runner.bench_time_func('ascii85_small', bench_a85_small)
    runner.bench_time_func('ascii85_large', bench_a85_large)

    runner.bench_time_func('base85_small', bench_b85_small)
    runner.bench_time_func('base85_large', bench_b85_large)
