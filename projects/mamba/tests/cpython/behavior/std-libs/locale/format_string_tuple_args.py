# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_tuple_args"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string with positional tuple args plus an escaped %% mid-string matches plain %-formatting"""
import locale

# Positional tuple args plus an escaped %% mid-string.
assert (
    locale.format_string("%d %f%%d", (1, 1.0))
    == "%d %f%%d" % (1, 1.0)
), "format_string tuple args"

print("format_string_tuple_args OK")
