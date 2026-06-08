# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "line_continuation_collapses"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: a line-continuation backslash collapses to nothing even inside an f-string: eval('f"\\\\\\n"') is ''"""
# a trailing backslash-newline is a line continuation

assert eval('f"\\\n"') == ""

print("line_continuation_collapses OK")
