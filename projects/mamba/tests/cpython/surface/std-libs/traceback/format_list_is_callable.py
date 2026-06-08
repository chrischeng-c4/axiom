# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "format_list_is_callable"
# subject = "traceback.format_list"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_list: format_list_is_callable (surface)."""
import traceback

assert callable(traceback.format_list)
print("format_list_is_callable OK")
