# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "error_subclasses_exception"
# subject = "binascii.Error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.Error: binascii.Error and binascii.Incomplete subclass Exception (Error is a ValueError)"""
import binascii

assert issubclass(binascii.Error, Exception), "Error subclasses Exception"
assert issubclass(binascii.Error, ValueError), "Error subclasses ValueError"
assert issubclass(binascii.Incomplete, Exception), "Incomplete subclasses Exception"

print("error_subclasses_exception OK")
