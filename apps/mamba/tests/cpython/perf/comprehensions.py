# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "comprehensions"
# dimension = "perf"
# case = "comprehensions"
# subject = "pyperformance comprehensions"
# kind = "bench"
# xfail = "mamba must run the pyperformance comprehensions workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_comprehensions/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance comprehensions workload faster than CPython on CPU+RSS
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
Benchmark comprehensions.

Author: Carl Meyer
"""

from dataclasses import dataclass
from enum import Enum
from typing import Iterable, List, Optional

import pyperf


class WidgetKind(Enum):
    BIG = 1
    SMALL = 2


@dataclass
class Widget:
    widget_id: int
    creator_id: int
    derived_widget_ids: List[int]
    kind: WidgetKind
    has_knob: bool
    has_spinner: bool


class WidgetTray:
    def __init__(self, owner_id: int, widgets: List[Widget]) -> None:
        self.owner_id = owner_id
        self.sorted_widgets: List[Widget] = []
        self._add_widgets(widgets)

    def _any_knobby(self, widgets: Iterable[Optional[Widget]]) -> bool:
        return any(w.has_knob for w in widgets if w)

    def _is_big_spinny(self, widget: Widget) -> bool:
        return widget.kind == WidgetKind.BIG and widget.has_spinner

    def _add_widgets(self, widgets: List[Widget]) -> None:
        # sort order: mine first, then any widgets with derived knobby widgets in order of
        # number derived, then other widgets in order of number derived, and we exclude
        # big spinny widgets entirely
        widgets = [w for w in widgets if not self._is_big_spinny(w)]
        id_to_widget = {w.widget_id: w for w in widgets}
        id_to_derived = {
            w.widget_id: [id_to_widget.get(dwid) for dwid in w.derived_widget_ids]
            for w in widgets
        }
        sortable_widgets = [
            (
                w.creator_id == self.owner_id,
                self._any_knobby(id_to_derived[w.widget_id]),
                len(id_to_derived[w.widget_id]),
                w.widget_id,
            )
            for w in widgets
        ]
        sortable_widgets.sort()
        self.sorted_widgets = [id_to_widget[sw[-1]] for sw in sortable_widgets]


def make_some_widgets() -> List[Widget]:
    widget_id = 0
    widgets = []
    for creator_id in range(3):
        for kind in WidgetKind:
            for has_knob in [True, False]:
                for has_spinner in [True, False]:
                    derived = [w.widget_id for w in widgets[::creator_id + 1]]
                    widgets.append(
                        Widget(
                            widget_id, creator_id, derived, kind, has_knob, has_spinner
                        )
                    )
                    widget_id += 1
    assert len(widgets) == 24
    return widgets


def bench_comprehensions(loops: int) -> float:
    range_it = range(loops)
    widgets = make_some_widgets()
    t0 = pyperf.perf_counter()
    for _ in range_it:
        tray = WidgetTray(1, widgets)
        assert len(tray.sorted_widgets) == 18
    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata["description"] = "Benchmark comprehensions"
    runner.bench_time_func("comprehensions", bench_comprehensions)
