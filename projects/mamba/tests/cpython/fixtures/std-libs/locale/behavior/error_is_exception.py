# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "error_is_exception"
# subject = "locale.Error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.Error: locale.Error is a subclass of Exception"""
import locale

assert issubclass(locale.Error, Exception), "locale.Error is an Exception subclass"

print("error_is_exception OK")
