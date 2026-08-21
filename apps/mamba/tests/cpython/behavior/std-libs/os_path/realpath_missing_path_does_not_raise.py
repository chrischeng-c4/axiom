# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "realpath_missing_path_does_not_raise"
# subject = "os.path.realpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.realpath: realpath on a non-existent path does not raise; it returns the canonicalized path string (here still rooted under '/no')"""
import os.path

_rp = os.path.realpath("/no/such/path")
assert _rp.startswith("/no"), f"realpath missing returns path = {_rp!r}"

print("realpath_missing_path_does_not_raise OK")
