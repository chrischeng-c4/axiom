# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "machine_is_callable"
# subject = "platform.machine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.machine: machine_is_callable (surface)."""
import platform

assert callable(platform.machine)
print("machine_is_callable OK")
