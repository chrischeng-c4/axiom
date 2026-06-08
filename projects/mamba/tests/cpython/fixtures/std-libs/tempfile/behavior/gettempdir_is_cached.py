# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdir_is_cached"
# subject = "tempfile.gettempdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdir: gettempdir() caches its result: repeat calls return the very same string object (is-identity)"""
import tempfile

_a = tempfile.gettempdir()
_b = tempfile.gettempdir()
assert _a is _b, "gettempdir is cached"
print("gettempdir_is_cached OK")
