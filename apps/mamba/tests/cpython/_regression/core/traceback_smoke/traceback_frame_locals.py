# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/traceback_smoke: traceback tb_frame.f_locals fidelity (#3069)."""

import sys
from typing import Any


def level3(c_local: int) -> None:
    deep: int = c_local * 2
    raise ValueError("boom")


def level2(b_local: int) -> None:
    mid: int = b_local + 100
    level3(mid)


def level1(a_local: int) -> None:
    top: int = a_local + 10
    level2(top)


try:
    level1(5)
except ValueError:
    tb: Any = sys.exc_info()[2]
    while tb is not None:
        f: Any = tb.tb_frame
        loc: Any = f.f_locals
        names: list = sorted([k for k in loc.keys() if not k.startswith("__")])
        ints: list = [loc[k] for k in names if isinstance(loc[k], int)]
        print(f"[traceback-locals] {f.f_code.co_name}", names, ints)
        tb = tb.tb_next

# Control case: locals() builtin behavior
def control_locals(a: int) -> None:
    b: int = a + 1
    print("[traceback-locals] locals():", sorted(locals().keys()))

control_locals(3)
