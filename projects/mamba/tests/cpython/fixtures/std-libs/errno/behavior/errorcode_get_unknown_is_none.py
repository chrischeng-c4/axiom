# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_get_unknown_is_none"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode.get on an errno number that is not present returns None (the dict .get default)"""
import errno

assert 99999 not in errno.errorcode, "99999 unexpectedly present in errorcode"
assert errno.errorcode.get(99999) is None, errno.errorcode.get(99999)
print("errorcode_get_unknown_is_none OK")
