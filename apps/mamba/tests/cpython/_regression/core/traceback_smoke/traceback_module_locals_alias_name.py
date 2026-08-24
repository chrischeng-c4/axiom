# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/traceback_smoke: <module> frame f_locals is name-agnostic (#3069).

The name a caller binds `tb_frame.f_locals` to is an ordinary module-level
assignment, so it appears in the module namespace like any other. Nothing about
CPython's behaviour depends on which name is chosen. A companion fixture,
traceback_frame_locals.py, happens to use `loc`; this one uses `snap` precisely
so that an implementation which special-cases a particular literal name cannot
satisfy both.
"""

import sys
from typing import Any


def level1(a_local: int) -> None:
    top: int = a_local + 10
    raise ValueError("boom")


try:
    level1(5)
except ValueError:
    tb: Any = sys.exc_info()[2]
    f: Any = tb.tb_frame
    snap: Any = f.f_locals
    names: list = sorted([k for k in snap.keys() if not k.startswith("__")])
    print("[alias-name] <module> names:", names)
    print("[alias-name] bound alias 'snap' present:", "snap" in names)
    print("[alias-name] unrelated name 'loc' absent:", "loc" not in names)
    assert "snap" in names, f"module f_locals must contain its own alias: {names}"
    assert "loc" not in names, f"module f_locals invented a name: {names}"
