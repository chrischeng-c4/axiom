# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "real_world"
# case = "translate_oserror_errno_to_name"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: a realistic flow: catch an OSError raised by opening a missing file, then translate its .errno to the symbolic name via errorcode and confirm it is ENOENT"""
import errno
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    missing = os.path.join(d, "does_not_exist.txt")
    caught = None
    try:
        open(missing)
    except OSError as exc:
        caught = exc

assert caught is not None, "opening a missing file did not raise OSError"
assert caught.errno == errno.ENOENT, caught.errno
assert errno.errorcode.get(caught.errno) == "ENOENT", errno.errorcode.get(caught.errno)
print("translate_oserror_errno_to_name OK")
