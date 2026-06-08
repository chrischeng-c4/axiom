# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "eagain_equals_ewouldblock"
# subject = "errno.EAGAIN"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.EAGAIN: EAGAIN and EWOULDBLOCK alias the same errno value (EAGAIN == EWOULDBLOCK)"""
import errno

assert errno.EAGAIN == errno.EWOULDBLOCK, (errno.EAGAIN, errno.EWOULDBLOCK)
print("eagain_equals_ewouldblock OK")
