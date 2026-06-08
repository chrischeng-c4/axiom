# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_key_value_types"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: every errorcode key is an int and every value is a str"""
import errno

assert errno.errorcode, "errorcode is unexpectedly empty"
assert all(isinstance(k, int) for k in errno.errorcode), "a key is not int"
assert all(isinstance(v, str) for v in errno.errorcode.values()), "a value is not str"
print("errorcode_key_value_types OK")
