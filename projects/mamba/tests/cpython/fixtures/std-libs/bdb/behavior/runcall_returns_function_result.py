# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runcall_returns_function_result"
# subject = "bdb.Bdb.runcall"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runcall: runcall runs a function under the debugger and returns its result (a Bdb subclass that set_continue()s on each line returns 42 from a lambda)"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runcall(lambda: 42)
assert _r == 42, f"runcall result = {_r!r}"

print("runcall_returns_function_result OK")
