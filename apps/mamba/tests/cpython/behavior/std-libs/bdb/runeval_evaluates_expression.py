# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runeval_evaluates_expression"
# subject = "bdb.Bdb.runeval"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runeval method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runeval: runeval evaluates an expression under the debugger and returns its value: runeval('1 + 2 + 3') is 6"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runeval("1 + 2 + 3")
assert _r == 6, f"runeval = {_r!r}"

print("runeval_evaluates_expression OK")
