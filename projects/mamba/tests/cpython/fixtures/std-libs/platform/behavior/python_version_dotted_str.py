# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_version_dotted_str"
# subject = "platform.python_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_version: python_version() returns a dotted version str that starts with '3.' under CPython 3.12"""
import platform

out = platform.python_version()
assert type(out).__name__ == "str", "python_version() returns str"
assert out.startswith("3."), "python_version() starts with '3.'"

print("python_version_dotted_str OK")
