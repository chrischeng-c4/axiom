# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_combines_segments"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: join glues path segments with the POSIX '/' separator across two-, three-, and four-segment inputs"""
import os.path

assert os.path.join("a", "b", "c") == "a/b/c", "join 3 segments"
assert os.path.join("a", "b", "c", "d") == "a/b/c/d", "four-segment join"
assert os.path.join("/usr", "local", "bin") == "/usr/local/bin", "join with root"
assert os.path.join("usr", "local", "bin", "python3") == "usr/local/bin/python3", "toolchain path"

print("join_combines_segments OK")
