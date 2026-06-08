# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "which_resolves_existing_executable"
# subject = "shutil.which"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.which: which('ls') resolves an on-PATH executable to an absolute path string (os.path.isabs)"""
import shutil
import os

# 'ls' is on PATH on macOS/Linux; which() resolves it to an absolute path.
ls = shutil.which("ls")
assert isinstance(ls, str), f"which('ls') = {ls!r}"
assert os.path.isabs(ls), f"which returns absolute = {ls!r}"

print("which_resolves_existing_executable OK")
