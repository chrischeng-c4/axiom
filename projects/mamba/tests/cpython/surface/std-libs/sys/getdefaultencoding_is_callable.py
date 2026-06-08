# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "getdefaultencoding_is_callable"
# subject = "sys.getdefaultencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getdefaultencoding: getdefaultencoding_is_callable (surface)."""
import sys

assert callable(sys.getdefaultencoding)
print("getdefaultencoding_is_callable OK")
