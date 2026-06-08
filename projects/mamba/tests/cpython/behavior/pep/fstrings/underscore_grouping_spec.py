# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "underscore_grouping_spec"
# subject = "fstring.format_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: the '_' grouping option inserts underscores every three digits: f'{1000000:_}' is '1_000_000'"""
# '_' is a thousands-grouping option in the format spec

assert f"{1000000:_}" == "1_000_000", "underscore sep"

print("underscore_grouping_spec OK")
