# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "lc_time_is_int"
# subject = "locale.LC_TIME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.LC_TIME: lc_time_is_int (surface)."""
import locale

assert type(locale.LC_TIME).__name__ == "int"
print("lc_time_is_int OK")
