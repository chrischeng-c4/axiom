# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getcwd_is_absolute"
# subject = "os.getcwd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwd: os.getcwd returns a non-empty absolute path string"""
import os
import os.path

cwd = os.getcwd()
assert isinstance(cwd, str), f"getcwd type = {type(cwd)!r}"
assert len(cwd) > 0, "getcwd non-empty"
assert os.path.isabs(cwd), f"cwd is absolute: {cwd!r}"
print("getcwd_is_absolute OK")
