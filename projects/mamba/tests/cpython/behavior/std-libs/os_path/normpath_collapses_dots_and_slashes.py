# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "normpath_collapses_dots_and_slashes"
# subject = "os.path.normpath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.normpath: normpath collapses '//', '.', and '..' segments; '/usr//local/../local/bin/./python' -> '/usr/local/bin/python', './a/../b' -> 'b', 'a/b/c/../../d' -> 'a/d', and '/' is unchanged"""
import os.path

assert os.path.normpath("/usr//local/../local/bin/./python") == "/usr/local/bin/python", "normpath absolute"
assert os.path.normpath("a/b/c/../d") == "a/b/d", "normpath relative"
assert os.path.normpath("/usr//local") == "/usr/local", "double slashes (single-slash prefix)"
assert os.path.normpath("./a/../b") == "b", "dot and dot-dot"
assert os.path.normpath("a/b/c/../../d") == "a/d", "multiple dot-dots"
assert os.path.normpath("/") == "/", "root unchanged"

print("normpath_collapses_dots_and_slashes OK")
