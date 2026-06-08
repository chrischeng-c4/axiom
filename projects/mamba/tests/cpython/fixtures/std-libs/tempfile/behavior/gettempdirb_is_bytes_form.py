# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdirb_is_bytes_form"
# subject = "tempfile.gettempdirb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdirb: gettempdirb() is the bytes form of gettempdir(): bytes type, and os.fsdecode of it equals the str form"""
import os
import tempfile

_a = tempfile.gettempdir()
_c = tempfile.gettempdirb()
assert isinstance(_c, bytes), f"gettempdirb type = {type(_c)!r}"
assert type(_a) is not type(_c), "str vs bytes are distinct types"
assert _a == os.fsdecode(_c), "decoded bytes form matches str form"
print("gettempdirb_is_bytes_form OK")
