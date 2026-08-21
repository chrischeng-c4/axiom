# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "python_implementation_is_callable"
# subject = "platform.python_implementation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.python_implementation: python_implementation_is_callable (surface)."""
import platform

assert callable(platform.python_implementation)
print("python_implementation_is_callable OK")
