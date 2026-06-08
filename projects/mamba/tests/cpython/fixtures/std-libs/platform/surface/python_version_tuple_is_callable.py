# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "python_version_tuple_is_callable"
# subject = "platform.python_version_tuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.python_version_tuple: python_version_tuple_is_callable (surface)."""
import platform

assert callable(platform.python_version_tuple)
print("python_version_tuple_is_callable OK")
