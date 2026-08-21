# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "std_c_errors_are_int"
# subject = "errno.EDOM"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.EDOM: the standard-C math errors EDOM and ERANGE are exposed as ints (not just POSIX errors are present)"""
import errno

for name in ("EDOM", "ERANGE"):
    assert hasattr(errno, name), f"errno is missing {name}"
    assert isinstance(getattr(errno, name), int), f"errno.{name} is not int"
print("std_c_errors_are_int OK")
