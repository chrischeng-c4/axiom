# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "commonpath_longest_shared_prefix"
# subject = "os.path.commonpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.commonpath: commonpath returns the longest shared directory prefix; commonpath(['/usr/local/bin','/usr/local/lib']) == '/usr/local'"""
import os.path

_cp = os.path.commonpath(["/usr/local/bin", "/usr/local/lib"])
assert _cp == "/usr/local", f"commonpath = {_cp!r}"

print("commonpath_longest_shared_prefix OK")
