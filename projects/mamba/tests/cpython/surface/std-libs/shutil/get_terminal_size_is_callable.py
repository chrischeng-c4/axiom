# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "get_terminal_size_is_callable"
# subject = "shutil.get_terminal_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.get_terminal_size: get_terminal_size_is_callable (surface)."""
import shutil

assert callable(shutil.get_terminal_size)
print("get_terminal_size_is_callable OK")
