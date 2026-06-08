"""Timer scheduling benchmark - 3-way comparison.

Measures call_later performance by scheduling many timers with varying
delays (0-9 ms) and waiting for all to fire.
"""

from __future__ import annotations

from cclab.probe import BenchmarkGroup, register_group

from ._helpers import BACKENDS, close_loop, create_loop

NUM_TIMERS = 1_000

group = BenchmarkGroup("call_later Timer Scheduling")


def _make_bench(backend: str):
    """Return a benchmark function for *backend*."""

    def bench():
        loop = create_loop(backend)
        completed = [0]

        def callback():
            completed[0] += 1
            if completed[0] >= NUM_TIMERS:
                loop.stop()

        for i in range(NUM_TIMERS):
            delay = (i % 10) * 0.001  # 0-9 ms
            loop.call_later(delay, callback)

        loop.run_forever()
        close_loop(loop, backend)

    return bench


for _backend in BACKENDS:
    group.add(_backend)(_make_bench(_backend))

register_group(group)
