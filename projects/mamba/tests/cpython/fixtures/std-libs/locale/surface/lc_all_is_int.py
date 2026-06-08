# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "lc_all_is_int"
# subject = "locale.LC_ALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.LC_ALL: lc_all_is_int (surface)."""
import locale

assert type(locale.LC_ALL).__name__ == "int"
print("lc_all_is_int OK")
