# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_none_outside_handler"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: outside any except block, exc_info() returns (None, None, None)"""
import sys

_exc_type, _exc_val, _exc_tb = sys.exc_info()
assert _exc_type is None, f"exc_type outside = {_exc_type!r}"
assert _exc_val is None, f"exc_val outside = {_exc_val!r}"
assert _exc_tb is None, f"exc_tb outside = {_exc_tb!r}"
print("exc_info_none_outside_handler OK")
