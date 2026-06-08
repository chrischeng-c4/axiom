# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "uname_is_callable"
# subject = "platform.uname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.uname: uname_is_callable (surface)."""
import platform

assert callable(platform.uname)
print("uname_is_callable OK")
