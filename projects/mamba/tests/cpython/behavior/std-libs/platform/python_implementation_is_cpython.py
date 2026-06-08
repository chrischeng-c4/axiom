# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_implementation_is_cpython"
# subject = "platform.python_implementation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_implementation: python_implementation() returns 'CPython' on the reference interpreter"""
import platform

assert platform.python_implementation() == "CPython", "running CPython"

print("python_implementation_is_cpython OK")
