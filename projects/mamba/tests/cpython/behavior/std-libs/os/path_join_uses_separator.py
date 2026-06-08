# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_join_uses_separator"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: os.path.join('a','b','c') joins with the platform separator so normalizing os.sep to '/' yields 'a/b/c'"""
import os
import os.path

joined = os.path.join("a", "b", "c")
assert isinstance(joined, str), f"join type = {type(joined)!r}"
# On Unix: "a/b/c"; normalizing the platform separator to '/' is portable.
assert joined.replace(os.sep, "/") == "a/b/c", f"join = {joined!r}"
print("path_join_uses_separator OK")
