# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "architecture_is_callable"
# subject = "platform.architecture"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.architecture: architecture_is_callable (surface)."""
import platform

assert callable(platform.architecture)
print("architecture_is_callable OK")
