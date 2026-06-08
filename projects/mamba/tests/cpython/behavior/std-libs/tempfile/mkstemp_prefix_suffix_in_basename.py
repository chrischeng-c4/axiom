# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_prefix_suffix_in_basename"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp(prefix=, suffix=) embeds the prefix and suffix in the basename of the returned path"""
import os
import tempfile

_fd, _path = tempfile.mkstemp(suffix=".txt", prefix="myapp_")
try:
    _base = os.path.basename(_path)
    assert _base.endswith(".txt"), f"suffix = {_base!r}"
    assert _base.startswith("myapp_"), f"prefix = {_base!r}"
finally:
    os.close(_fd)
    os.unlink(_path)
print("mkstemp_prefix_suffix_in_basename OK")
