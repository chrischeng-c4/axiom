# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "logging"
# dimension = "perf"
# case = "logging"
# subject = "pyperformance logging"
# kind = "bench"
# xfail = "mamba must run the pyperformance logging workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_logging/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance logging workload faster than CPython on CPU+RSS
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
Script for testing the performance of logging simple messages.

Rationale for logging_silent by Antoine Pitrou:

"The performance of silent logging calls is actually important for all
applications which have debug() calls in their critical paths.  This is
quite common in network and/or distributed programming where you want to
allow logging many events for diagnosis of unexpected runtime issues
(because many unexpected conditions can appear), but with those logs
disabled by default for performance and readability reasons."

https://mail.python.org/pipermail/speed/2017-May/000576.html
"""

# Python imports
import io
import logging

# Third party imports
import pyperf

# A simple format for parametered logging
FORMAT = 'important: %s'
MESSAGE = 'some important information to be logged'


def truncate_stream(stream):
    stream.seek(0)
    stream.truncate()


def bench_silent(loops, logger, stream):
    truncate_stream(stream)

    # micro-optimization: use fast local variables
    m = MESSAGE
    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for _ in range_it:
        # repeat 10 times
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)
        logger.debug(m)

    dt = pyperf.perf_counter() - t0

    if len(stream.getvalue()) != 0:
        raise ValueError("stream is expected to be empty")

    return dt


def bench_simple_output(loops, logger, stream):
    truncate_stream(stream)

    # micro-optimization: use fast local variables
    m = MESSAGE
    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for _ in range_it:
        # repeat 10 times
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)
        logger.warning(m)

    dt = pyperf.perf_counter() - t0

    lines = stream.getvalue().splitlines()
    if len(lines) != loops * 10:
        raise ValueError("wrong number of lines")

    return dt


def bench_formatted_output(loops, logger, stream):
    truncate_stream(stream)

    # micro-optimization: use fast local variables
    fmt = FORMAT
    msg = MESSAGE
    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for _ in range_it:
        # repeat 10 times
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)
        logger.warning(fmt, msg)

    dt = pyperf.perf_counter() - t0

    lines = stream.getvalue().splitlines()
    if len(lines) != loops * 10:
        raise ValueError("wrong number of lines")

    return dt


def add_cmdline_args(cmd, args):
    if args.benchmark:
        cmd.append(args.benchmark)


BENCHMARKS = {
    "silent": bench_silent,
    "simple": bench_simple_output,
    "format": bench_formatted_output,
}


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata['description'] = "Test the performance of logging."

    parser = runner.argparser
    parser.add_argument("benchmark", nargs='?', choices=sorted(BENCHMARKS))

    options = runner.parse_args()

    # NOTE: StringIO performance will impact the results...
    stream = io.StringIO()

    handler = logging.StreamHandler(stream=stream)
    logger = logging.getLogger("benchlogger")
    logger.propagate = False
    logger.addHandler(handler)
    logger.setLevel(logging.WARNING)

    if options.benchmark:
        benchmarks = (options.benchmark,)
    else:
        benchmarks = sorted(BENCHMARKS)

    for bench in benchmarks:
        name = 'logging_%s' % bench
        bench_func = BENCHMARKS[bench]

        runner.bench_time_func(name, bench_func,
                               logger, stream,
                               inner_loops=10)
