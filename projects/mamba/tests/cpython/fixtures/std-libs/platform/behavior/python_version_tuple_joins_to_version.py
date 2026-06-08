# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "python_version_tuple_joins_to_version"
# subject = "platform.python_version_tuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.python_version_tuple: python_version_tuple() yields a 3-part tuple of digit strings whose '.'-join equals python_version()"""
import platform

parts = platform.python_version_tuple()
assert len(parts) == 3, "version tuple has 3 parts"
assert ".".join(parts) == platform.python_version(), "tuple joins to version"
assert all(p.isdigit() for p in parts[:2]), "major/minor are digits"

print("python_version_tuple_joins_to_version OK")
