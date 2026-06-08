# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "surface"
# case = "ascii_builtin_is_callable"
# subject = "ascii"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ascii: ascii_builtin_is_callable (surface)."""
# the !a conversion flag dispatches via the ascii() builtin

assert callable(ascii)
print("ascii_builtin_is_callable OK")
