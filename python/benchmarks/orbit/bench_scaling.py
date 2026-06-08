"""Parametrised scaling analysis benchmark - 3-way comparison.

Programmatically creates benchmark groups at different scales:
- Callback scaling: 1K, 10K, 50K iterations
- Timer scaling: 100, 1K, 5K timers
"""

from __future__ import annotations

from cclab.probe import BenchmarkGroup, register_group

from ._helpers import BACKENDS, close_loop, create_loop

# ---------------------------------------------------------------------------
# Callback Scaling
# ---------------------------------------------------------------------------

CALLBACK_SCALES = [1_000, 10_000, 50_000]


def _make_callback_bench(backend: str, n: int):
    def bench():
        loop = create_loop(backend)
        counter = [0]

        def callback():
            counter[0] += 1
            if counter[0] < n:
                loop.call_soon(callback)
            else:
                loop.stop()

        loop.call_soon(callback)
        loop.run_forever()
        close_loop(loop, backend)

    return bench


for _n in CALLBACK_SCALES:
    _group = BenchmarkGroup(f"Callback Scaling ({_n:,})")
    for _backend in BACKENDS:
        _group.add(_backend)(_make_callback_bench(_backend, _n))
    register_group(_group)

# ---------------------------------------------------------------------------
# Timer Scaling
# ---------------------------------------------------------------------------

TIMER_SCALES = [100, 1_000, 5_000]


def _make_timer_bench(backend: str, n: int):
    def bench():
        loop = create_loop(backend)
        completed = [0]

        def callback():
            completed[0] += 1
            if completed[0] >= n:
                loop.stop()

        for i in range(n):
            delay = (i % 10) * 0.001
            loop.call_later(delay, callback)

        loop.run_forever()
        close_loop(loop, backend)

    return bench


for _n in TIMER_SCALES:
    _group = BenchmarkGroup(f"Timer Scaling ({_n:,})")
    for _backend in BACKENDS:
        _group.add(_backend)(_make_timer_bench(_backend, _n))
    register_group(_group)
