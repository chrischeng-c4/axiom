# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "doubled_braces_are_literal"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: doubled braces produce a single literal brace and never start a field: f'{{1+1}}' is '{1+1}' and f'{{1+1' is '{1+1'"""
# {{ and }} escape to literal braces

assert f"{{1+1}}" == "{1+1}"
assert f"{{1+1" == "{1+1"

print("doubled_braces_are_literal OK")
