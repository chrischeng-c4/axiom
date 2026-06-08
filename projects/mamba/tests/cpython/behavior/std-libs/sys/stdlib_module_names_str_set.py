# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "stdlib_module_names_str_set"
# subject = "sys.stdlib_module_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.stdlib_module_names: stdlib_module_names is a frozenset of str naming bundled modules and contains 'sys'"""
import sys

assert isinstance(sys.stdlib_module_names, frozenset), \
    f"stdlib_module_names type = {type(sys.stdlib_module_names)!r}"
assert all(isinstance(n, str) for n in sys.stdlib_module_names), \
    "stdlib_module_names entries are all str"
assert "sys" in sys.stdlib_module_names, "sys names itself as stdlib"
print("stdlib_module_names_str_set OK")
