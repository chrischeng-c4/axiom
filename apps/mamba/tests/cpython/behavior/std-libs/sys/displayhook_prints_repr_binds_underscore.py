# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "displayhook_prints_repr_binds_underscore"
# subject = "sys.__displayhook__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: __displayhook__(42) writes '42\\n' to stdout and binds builtins._ to 42 for a non-None value"""
import builtins
import io
import sys
from contextlib import redirect_stdout

buf = io.StringIO()
with redirect_stdout(buf):
    sys.__displayhook__(42)
assert buf.getvalue() == "42\n", f"displayhook(42) wrote {buf.getvalue()!r}"
assert builtins._ == 42, f"builtins._ = {builtins._!r}"
del builtins._
print("displayhook_prints_repr_binds_underscore OK")
