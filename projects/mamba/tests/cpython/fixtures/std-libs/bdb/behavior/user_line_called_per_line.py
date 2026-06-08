# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "user_line_called_per_line"
# subject = "bdb.Bdb.user_line"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/user_line tracing (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.user_line: the debugger invokes user_line at least once while tracing a multi-statement function, and runcall still returns the correct result"""
import bdb


class _Dbg(bdb.Bdb):
    def __init__(self):
        super().__init__()
        self.line_count = 0

    def user_line(self, frame):
        self.line_count += 1
        if self.line_count >= 5:
            self.set_continue()


def _simple():
    x = 1
    y = x + 1
    z = y + 1
    return z


_d = _Dbg()
_r = _d.runcall(_simple)
assert _r == 3, f"_simple result = {_r!r}"
assert _d.line_count > 0, "user_line was called"

print("user_line_called_per_line OK")
