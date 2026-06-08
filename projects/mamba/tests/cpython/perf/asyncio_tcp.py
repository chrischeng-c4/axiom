# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "asyncio_tcp"
# dimension = "perf"
# case = "asyncio_tcp"
# subject = "pyperformance asyncio_tcp"
# kind = "bench"
# xfail = "mamba must run the pyperformance asyncio_tcp workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_asyncio_tcp/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance asyncio_tcp workload faster than CPython on CPU+RSS
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
Benchmark for asyncio TCP server and client performance
transferring 10MB of data.

Author: Kumar Aditya
"""


import asyncio
from pyperf import Runner
import ssl
import os

CHUNK_SIZE = 1024 ** 2 * 10
# Taken from CPython's test suite
SSL_CERT = os.path.join(os.path.dirname(__file__), 'ssl_cert.pem')
SSL_KEY = os.path.join(os.path.dirname(__file__), 'ssl_key.pem')


async def handle_echo(reader: asyncio.StreamReader,
                      writer: asyncio.StreamWriter) -> None:
    data = b'x' * CHUNK_SIZE
    for _ in range(100):
        writer.write(data)
        await writer.drain()
    writer.close()
    await writer.wait_closed()


async def main(use_ssl: bool) -> None:
    if use_ssl:
        server_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        server_context.load_cert_chain(SSL_CERT, SSL_KEY)
        server_context.check_hostname = False
        server_context.verify_mode = ssl.CERT_NONE

        client_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        client_context.load_cert_chain(SSL_CERT, SSL_KEY)
        client_context.check_hostname = False
        client_context.verify_mode = ssl.CERT_NONE
    else:
        server_context = None
        client_context = None

    server = await asyncio.start_server(handle_echo, '127.0.0.1', 8882, ssl=server_context)

    async with server:
        asyncio.create_task(server.start_serving())
        reader, writer = await asyncio.open_connection('127.0.0.1', 8882, ssl=client_context)
        data_len = 0
        while True:
            data = await reader.read(CHUNK_SIZE)
            if not data:
                break
            data_len += len(data)
        assert data_len == CHUNK_SIZE * 100
        writer.close()
        await writer.wait_closed()


def add_cmdline_args(cmd, args):
    if args.ssl:
        cmd.append("--ssl")


if __name__ == '__main__':
    runner = Runner(add_cmdline_args=add_cmdline_args)
    parser = runner.argparser
    parser.add_argument('--ssl', action='store_true', default=False)
    args = runner.parse_args()
    name = 'asyncio_tcp' + ('_ssl' if args.ssl else '')
    runner.bench_async_func(name, main, args.ssl)
