# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "implementation_describes_interpreter"
# subject = "sys.implementation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.implementation: sys.implementation has a lowercase name, a version whose first two fields index as (major, minor), an int hexversion, and a str cache_tag"""
import sys

assert sys.implementation.name == sys.implementation.name.lower(), \
    f"implementation.name = {sys.implementation.name!r}"
_iv = sys.implementation.version
assert _iv[:2] == (_iv.major, _iv.minor), "implementation.version indexing"
assert isinstance(sys.implementation.hexversion, int), "hexversion is int"
assert isinstance(sys.implementation.cache_tag, str), "cache_tag is str"
print("implementation_describes_interpreter OK")
