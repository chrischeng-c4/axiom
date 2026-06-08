# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_prefix_pattern"
# subject = "fnmatch.filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter(['Python','Ruby','Perl','Tcl'], 'P*') keeps the names starting with P in order: ['Python','Perl']"""
import fnmatch

_out = fnmatch.filter(["Python", "Ruby", "Perl", "Tcl"], "P*")
assert _out == ["Python", "Perl"], f"prefix filter = {_out!r}"

print("filter_prefix_pattern OK")
