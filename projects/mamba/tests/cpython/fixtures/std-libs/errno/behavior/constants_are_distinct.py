# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "constants_are_distinct"
# subject = "errno.EACCES"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EACCES: distinct named errors carry distinct values: EACCES != ENOENT"""
import errno

assert errno.EACCES != errno.ENOENT, (errno.EACCES, errno.ENOENT)
print("constants_are_distinct OK")
