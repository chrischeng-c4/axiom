# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "utils_formataddr_callable"
# subject = "email.utils.formataddr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.utils.formataddr: utils_formataddr_callable (surface)."""
import email.utils

assert callable(email.utils.formataddr)
print("utils_formataddr_callable OK")
