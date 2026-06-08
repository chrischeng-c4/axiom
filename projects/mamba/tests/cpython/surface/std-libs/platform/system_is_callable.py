# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "system_is_callable"
# subject = "platform.system"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.system: system_is_callable (surface)."""
import platform

assert callable(platform.system)
print("system_is_callable OK")
