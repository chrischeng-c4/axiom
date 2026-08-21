# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getdefaultencoding_is_utf8"
# subject = "sys.getdefaultencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getdefaultencoding: sys.getdefaultencoding() returns 'utf-8' on CPython 3.12"""
import sys

assert sys.getdefaultencoding() == "utf-8", \
    f"getdefaultencoding = {sys.getdefaultencoding()!r}"
print("getdefaultencoding_is_utf8 OK")
