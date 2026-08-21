# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "format_string_percent_escape"
# subject = "locale.format_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.format_string: format_string preserves a literal %% escape and formats a single float arg like %-formatting"""
import locale

# Literal %% survives, and a single float arg formats like %-formatting.
assert locale.format_string("%f%%", 1.0) == "%f%%" % 1.0, "percent escape, single arg"
assert locale.format_string("%f%%", 1.0) == "1.000000%", "percent escape value"

print("format_string_percent_escape OK")
