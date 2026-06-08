# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "sys_version_malformed_banner_raises"
# subject = "platform._sys_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._sys_version: sys_version_malformed_banner_raises (errors)."""
import platform

_raised = False
try:
    platform._sys_version("2. 4.3 (truncation) \\n[GCC]")
except ValueError:
    _raised = True
assert _raised, "sys_version_malformed_banner_raises: expected ValueError"
print("sys_version_malformed_banner_raises OK")
