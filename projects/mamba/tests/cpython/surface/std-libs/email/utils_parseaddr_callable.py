# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "utils_parseaddr_callable"
# subject = "email.utils.parseaddr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.utils.parseaddr: utils_parseaddr_callable (surface)."""
import email.utils

assert callable(email.utils.parseaddr)
print("utils_parseaddr_callable OK")
