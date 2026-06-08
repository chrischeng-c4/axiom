# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runcall_passes_arguments"
# subject = "bdb.Bdb.runcall"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runcall: runcall forwards positional arguments to the traced function: runcall(add, 10, 20) returns 30"""
import bdb


def _add(a, b):
    return a + b


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runcall(_add, 10, 20)
assert _r == 30, f"runcall with args = {_r!r}"

print("runcall_passes_arguments OK")
