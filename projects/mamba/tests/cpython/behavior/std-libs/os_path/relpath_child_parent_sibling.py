# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "relpath_child_parent_sibling"
# subject = "os.path.relpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.relpath: relpath walks the tree; child '/usr/local/bin' from '/usr/local' is 'bin', parent is '..', and sibling '/usr/lib' from '/usr/local/bin' is '../../lib'"""
import os.path

_rel = os.path.relpath("/usr/local/bin", "/usr/local")
assert _rel == "bin", f"relpath child = {_rel!r}"
_rel2 = os.path.relpath("/usr/local", "/usr/local/bin")
assert _rel2 == "..", f"relpath parent = {_rel2!r}"
_rel3 = os.path.relpath("/usr/lib", "/usr/local/bin")
assert _rel3 == "../../lib", f"relpath sibling = {_rel3!r}"

print("relpath_child_parent_sibling OK")
