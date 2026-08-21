# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "abspath_returns_absolute_path"
# subject = "os.path.abspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.abspath: abspath('.') yields a path that isabs() reports as absolute (cwd-dependent, so only the absoluteness is asserted)"""
import os.path

_abs = os.path.abspath(".")
assert os.path.isabs(_abs), f"abspath is absolute = {_abs!r}"

print("abspath_returns_absolute_path OK")
