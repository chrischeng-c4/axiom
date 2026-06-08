# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "error_is_exception_subclass"
# subject = "locale.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.Error: error_is_exception_subclass (surface)."""
import locale

assert hasattr(locale.Error, "args")
print("error_is_exception_subclass OK")
