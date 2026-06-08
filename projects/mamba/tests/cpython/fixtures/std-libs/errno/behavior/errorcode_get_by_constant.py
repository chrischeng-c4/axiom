# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_get_by_constant"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode can be looked up by a named constant: errorcode.get(errno.EACCES) == 'EACCES' and errorcode[errno.ENOENT] == 'ENOENT'"""
import errno

assert errno.errorcode.get(errno.EACCES) == "EACCES", errno.errorcode.get(errno.EACCES)
assert errno.errorcode[errno.ENOENT] == "ENOENT", errno.errorcode[errno.ENOENT]
print("errorcode_get_by_constant OK")
