# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempprefix_nonempty_str_and_bytes"
# subject = "tempfile.gettempprefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempprefix: gettempprefix() is a non-empty str and gettempprefixb() is the corresponding non-empty bytes"""
import tempfile

_p = tempfile.gettempprefix()
_pb = tempfile.gettempprefixb()
assert isinstance(_p, str) and len(_p) > 0, f"prefix = {_p!r}"
assert isinstance(_pb, bytes) and len(_pb) > 0, f"prefixb = {_pb!r}"
print("gettempprefix_nonempty_str_and_bytes OK")
