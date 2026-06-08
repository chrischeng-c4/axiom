# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "system_alias_no_args_raises"
# subject = "platform.system_alias"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.system_alias: system_alias_no_args_raises (errors)."""
import platform

_raised = False
try:
    platform.system_alias()
except TypeError:
    _raised = True
assert _raised, "system_alias_no_args_raises: expected TypeError"
print("system_alias_no_args_raises OK")
