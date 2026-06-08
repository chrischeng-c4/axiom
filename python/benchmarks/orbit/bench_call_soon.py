"""call_soon throughput benchmark - 3-way comparison.

Measures core callback scheduling performance by chaining call_soon
callbacks until a target iteration count is reached.
"""

from __future__ import annotations

from cclab.probe import BenchmarkGroup, register_group

from ._helpers import BACKENDS, close_loop, create_loop

ITERATIONS = 10_000

group = BenchmarkGroup("call_soon Throughput")


def _make_bench(backend: str):
    """Return a benchmark function for *backend*."""

    def bench():
        loop = create_loop(backend)
        counter = [0]

        def callback():
            counter[0] += 1
            if counter[0] < ITERATIONS:
                loop.call_soon(callback)
            else:
                loop.stop()

        loop.call_soon(callback)
        loop.run_forever()
        close_loop(loop, backend)

    return bench


for _backend in BACKENDS:
    group.add(_backend)(_make_bench(_backend))

register_group(group)
