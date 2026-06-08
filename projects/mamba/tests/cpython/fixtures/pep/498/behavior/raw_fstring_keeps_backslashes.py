# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "raw_fstring_keeps_backslashes"
# subject = "fstring.prefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.prefix: raw f-strings keep backslashes verbatim: rf'\\n' is '\\\\n' and rf'{1}\\t' is '1\\\\t'"""
# the r prefix disables backslash interpretation in the literal run

assert rf"\n" == "\\n"
assert rf"{1}\t" == "1\\t"

print("raw_fstring_keeps_backslashes OK")
