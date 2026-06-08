# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "version_is_callable"
# subject = "platform.version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.version: version_is_callable (surface)."""
import platform

assert callable(platform.version)
print("version_is_callable OK")
