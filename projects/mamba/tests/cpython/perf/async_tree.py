# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "async_tree"
# dimension = "perf"
# case = "async_tree"
# subject = "pyperformance async_tree"
# kind = "bench"
# xfail = "mamba must run the pyperformance async_tree workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_async_tree/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance async_tree workload faster than CPython on CPU+RSS
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
Benchmark for async tree workload, which calls asyncio.gather() on a tree
(6 levels deep, 6 branches per level) with the leaf nodes simulating some
(potentially) async work (depending on the benchmark variant). Benchmark
variants include:

1) "none": No actual async work in the async tree.
2) "io": All leaf nodes simulate async IO workload (async sleep 50ms).
3) "memoization": All leaf nodes simulate async IO workload with 90% of
                  the data memoized
4) "cpu_io_mixed": Half of the leaf nodes simulate CPU-bound workload and
                   the other half simulate the same workload as the
                   "memoization" variant.

All variants also have an "eager" flavor that uses the asyncio eager task
factory (if available), and a "tg" variant that uses TaskGroups.
"""


import asyncio
import math
import random

import pyperf


NUM_RECURSE_LEVELS = 6
NUM_RECURSE_BRANCHES = 6
RANDOM_SEED = 0
IO_SLEEP_TIME = 0.05
MEMOIZABLE_PERCENTAGE = 90
CPU_PROBABILITY = 0.5
FACTORIAL_N = 500


class AsyncTree:
    def __init__(self, use_task_groups=False):
        self.cache = {}
        self.use_task_groups = use_task_groups
        # set to deterministic random, so that the results are reproducible
        random.seed(RANDOM_SEED)

    async def mock_io_call(self):
        await asyncio.sleep(IO_SLEEP_TIME)

    async def workload_func(self):
        raise NotImplementedError(
            "To be implemented by each variant's derived class."
        )

    async def recurse_with_gather(self, recurse_level):
        if recurse_level == 0:
            await self.workload_func()
            return

        await asyncio.gather(
            *[self.recurse_with_gather(recurse_level - 1)
              for _ in range(NUM_RECURSE_BRANCHES)]
        )

    async def recurse_with_task_group(self, recurse_level):
        if recurse_level == 0:
            await self.workload_func()
            return

        async with asyncio.TaskGroup() as tg:
            for _ in range(NUM_RECURSE_BRANCHES):
                tg.create_task(
                    self.recurse_with_task_group(recurse_level - 1))

    async def run(self):
        if self.use_task_groups:
            await self.recurse_with_task_group(NUM_RECURSE_LEVELS)
        else:
            await self.recurse_with_gather(NUM_RECURSE_LEVELS)


class EagerMixin:
    async def run(self):
        loop = asyncio.get_running_loop()
        if hasattr(asyncio, 'eager_task_factory'):
            loop.set_task_factory(asyncio.eager_task_factory)
        return await super().run()


class NoneAsyncTree(AsyncTree):
    async def workload_func(self):
        return


class EagerAsyncTree(EagerMixin, NoneAsyncTree):
    pass


class IOAsyncTree(AsyncTree):
    async def workload_func(self):
        await self.mock_io_call()


class EagerIOAsyncTree(EagerMixin, IOAsyncTree):
    pass


class MemoizationAsyncTree(AsyncTree):
    async def workload_func(self):
        # deterministic random, seed set in AsyncTree.__init__()
        data = random.randint(1, 100)

        if data <= MEMOIZABLE_PERCENTAGE:
            if self.cache.get(data):
                return data

            self.cache[data] = True

        await self.mock_io_call()
        return data


class EagerMemoizationAsyncTree(EagerMixin, MemoizationAsyncTree):
    pass


class CpuIoMixedAsyncTree(MemoizationAsyncTree):
    async def workload_func(self):
        # deterministic random, seed set in AsyncTree.__init__()
        if random.random() < CPU_PROBABILITY:
            # mock cpu-bound call
            return math.factorial(FACTORIAL_N)
        else:
            return await MemoizationAsyncTree.workload_func(self)


class EagerCpuIoMixedAsyncTree(EagerMixin, CpuIoMixedAsyncTree):
    pass


def add_metadata(runner):
    runner.metadata["description"] = "Async tree workloads."
    runner.metadata["async_tree_recurse_levels"] = NUM_RECURSE_LEVELS
    runner.metadata["async_tree_recurse_branches"] = NUM_RECURSE_BRANCHES
    runner.metadata["async_tree_random_seed"] = RANDOM_SEED
    runner.metadata["async_tree_io_sleep_time"] = IO_SLEEP_TIME
    runner.metadata["async_tree_memoizable_percentage"] = MEMOIZABLE_PERCENTAGE
    runner.metadata["async_tree_cpu_probability"] = CPU_PROBABILITY
    runner.metadata["async_tree_factorial_n"] = FACTORIAL_N


def add_cmdline_args(cmd, args):
    cmd.append(args.benchmark)
    if args.task_groups:
        cmd.append("--task-groups")


def add_parser_args(parser):
    parser.add_argument(
        "benchmark",
        choices=BENCHMARKS,
        help="""\
Determines which benchmark to run. Options:
1) "none": No actual async work in the async tree.
2) "io": All leaf nodes simulate async IO workload (async sleep 50ms).
3) "memoization": All leaf nodes simulate async IO workload with 90%% of
                  the data memoized
4) "cpu_io_mixed": Half of the leaf nodes simulate CPU-bound workload and
                   the other half simulate the same workload as the
                   "memoization" variant.
""",
    )
    parser.add_argument(
        "--task-groups",
        action="store_true",
        default=False,
        help="Use TaskGroups instead of gather.",
    )


BENCHMARKS = {
    "none": NoneAsyncTree,
    "eager": EagerAsyncTree,
    "io": IOAsyncTree,
    "eager_io": EagerIOAsyncTree,
    "memoization": MemoizationAsyncTree,
    "eager_memoization": EagerMemoizationAsyncTree,
    "cpu_io_mixed": CpuIoMixedAsyncTree,
    "eager_cpu_io_mixed": EagerCpuIoMixedAsyncTree,
}


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    add_metadata(runner)
    add_parser_args(runner.argparser)
    args = runner.parse_args()
    benchmark = args.benchmark

    async_tree_class = BENCHMARKS[benchmark]
    async_tree = async_tree_class(use_task_groups=args.task_groups)
    bench_name = f"async_tree_{benchmark}"
    if args.task_groups:
        bench_name += "_tg"
    runner.bench_async_func(bench_name, async_tree.run)
