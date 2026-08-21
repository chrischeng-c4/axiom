# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_maps_int_to_name"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode is a dict mapping the errno int to its uppercase name: errorcode[1]=='EPERM', errorcode[2]=='ENOENT', errorcode[13]=='EACCES'"""
import errno

assert errno.errorcode[1] == "EPERM", errno.errorcode[1]
assert errno.errorcode[2] == "ENOENT", errno.errorcode[2]
assert errno.errorcode[13] == "EACCES", errno.errorcode[13]
print("errorcode_maps_int_to_name OK")
