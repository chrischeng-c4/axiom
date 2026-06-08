# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "lc_numeric_is_int"
# subject = "locale.LC_NUMERIC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.LC_NUMERIC: lc_numeric_is_int (surface)."""
import locale

assert type(locale.LC_NUMERIC).__name__ == "int"
print("lc_numeric_is_int OK")
