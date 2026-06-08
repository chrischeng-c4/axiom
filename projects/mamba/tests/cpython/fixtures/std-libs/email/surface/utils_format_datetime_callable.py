# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "utils_format_datetime_callable"
# subject = "email.utils.format_datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.utils.format_datetime: utils_format_datetime_callable (surface)."""
import email.utils

assert callable(email.utils.format_datetime)
print("utils_format_datetime_callable OK")
