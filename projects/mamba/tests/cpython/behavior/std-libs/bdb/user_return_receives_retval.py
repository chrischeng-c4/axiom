# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "user_return_receives_retval"
# subject = "bdb.Bdb.user_return"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/user_return tracing (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.user_return: user_return fires when a traced function returns and receives the return value (the recorded returns list contains the function's result)"""
import bdb


class _Dbg(bdb.Bdb):
    def __init__(self):
        super().__init__()
        self.returns = []

    def user_line(self, frame):
        self.set_step()

    def user_return(self, frame, retval):
        self.returns.append(retval)
        self.set_continue()


def _fn():
    return "result"


_d = _Dbg()
_r = _d.runcall(_fn)
assert _r == "result", f"runcall = {_r!r}"
assert "result" in _d.returns, f"user_return called: {_d.returns!r}"

print("user_return_receives_retval OK")
