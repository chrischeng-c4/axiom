# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "displayhook_none_writes_nothing"
# subject = "sys.__displayhook__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: __displayhook__(None) writes nothing and leaves builtins._ unset"""
import builtins
import io
import sys
from contextlib import redirect_stdout

if hasattr(builtins, "_"):
    del builtins._
buf = io.StringIO()
with redirect_stdout(buf):
    sys.__displayhook__(None)
assert buf.getvalue() == "", f"displayhook(None) wrote {buf.getvalue()!r}"
assert not hasattr(builtins, "_"), "displayhook(None) left builtins._ unset"
print("displayhook_none_writes_nothing OK")
