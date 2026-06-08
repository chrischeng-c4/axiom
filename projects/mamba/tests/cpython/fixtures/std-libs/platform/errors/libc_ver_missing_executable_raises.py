# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "libc_ver_missing_executable_raises"
# subject = "platform.libc_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.libc_ver: libc_ver_missing_executable_raises (errors)."""
import platform

_raised = False
try:
    platform.libc_ver("/no/such/exe")
except FileNotFoundError:
    _raised = True
assert _raised, "libc_ver_missing_executable_raises: expected FileNotFoundError"
print("libc_ver_missing_executable_raises OK")
