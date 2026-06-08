# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "posix_constant_values"
# subject = "errno.EPERM"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EPERM: the POSIX-stable low constants pin to their documented numbers: EPERM==1, ENOENT==2, EBADF==9, EACCES==13"""
import errno

assert errno.EPERM == 1, errno.EPERM
assert errno.ENOENT == 2, errno.ENOENT
assert errno.EBADF == 9, errno.EBADF
assert errno.EACCES == 13, errno.EACCES
print("posix_constant_values OK")
