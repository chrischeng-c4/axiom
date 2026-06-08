# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "python_compiler_is_callable"
# subject = "platform.python_compiler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.python_compiler: python_compiler_is_callable (surface)."""
import platform

assert callable(platform.python_compiler)
print("python_compiler_is_callable OK")
