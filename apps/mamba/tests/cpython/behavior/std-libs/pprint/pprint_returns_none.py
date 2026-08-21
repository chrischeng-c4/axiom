# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "pprint_returns_none"
# subject = "pprint.pprint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pprint: pprint() prints as a side effect and returns None (the return contract, independent of stream)"""
import contextlib
import io
import pprint

# pprint is a side-effecting printer: it writes to the stream and returns None.
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    rv = pprint.pprint(7)
assert rv is None
assert buf.getvalue() == "7\n", repr(buf.getvalue())
print("pprint_returns_none OK")
