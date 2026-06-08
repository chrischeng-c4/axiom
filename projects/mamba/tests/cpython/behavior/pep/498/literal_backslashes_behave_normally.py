# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "literal_backslashes_behave_normally"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: backslashes in the literal portion behave like a normal string: f'\\\\' is '\\\\' and f'\\\\\\\\' is '\\\\\\\\'"""
# backslash escapes in literal runs match plain string literals

assert f"\\" == "\\"
assert f"\\\\" == "\\\\"

print("literal_backslashes_behave_normally OK")
