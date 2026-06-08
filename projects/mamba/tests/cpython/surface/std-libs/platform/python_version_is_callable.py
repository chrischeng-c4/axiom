# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "python_version_is_callable"
# subject = "platform.python_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.python_version: python_version_is_callable (surface)."""
import platform

assert callable(platform.python_version)
print("python_version_is_callable OK")
