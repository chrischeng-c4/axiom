# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_quit_aborts_and_returns_none"
# subject = "bdb.Bdb.set_quit"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/set_quit (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_quit: set_quit() during tracing aborts the run (runcall returns None) and sets the quitting flag True"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_quit()


def _long_fn():
    a = 1
    b = 2
    c = 3
    return a + b + c


_d = _Dbg()
_r = _d.runcall(_long_fn)
assert _r is None, f"runcall returns None on set_quit: {_r!r}"
assert _d.quitting is True, "quitting flag set after set_quit"

print("set_quit_aborts_and_returns_none OK")
