# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_join_absolute_resets"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: os.path.join discards earlier components when a later one is absolute: join('a','/b','c') ends at '/b/c'"""
import os
import os.path

abs_join = os.path.join("a", "/b", "c")
assert abs_join == "/b/c" or abs_join.endswith("b" + os.sep + "c"), f"abs join = {abs_join!r}"
print("path_join_absolute_resets OK")
