# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "doubled_braces_are_literal"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: doubled braces produce single literal braces and never start a field: f'{{not interpolated}}' is '{not interpolated}'"""
# {{ and }} escape to literal braces

assert f"{{not interpolated}}" == "{not interpolated}", "escaped braces"

print("doubled_braces_are_literal OK")
