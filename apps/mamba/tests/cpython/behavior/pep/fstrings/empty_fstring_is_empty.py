# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "empty_fstring_is_empty"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: an f-string with no fields and no text is the empty string: f'' is ''"""
# an f-string with no content is an ordinary empty literal

assert f"" == "", "empty f-string"

print("empty_fstring_is_empty OK")
