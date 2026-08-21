# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "which_missing_returns_none"
# subject = "shutil.which"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.which: which() returns None for a command name that does not exist on PATH"""
import shutil

w = shutil.which("definitely_not_a_real_command_xyz_abc")
assert w is None, f"which nonexistent = {w!r}"

print("which_missing_returns_none OK")
